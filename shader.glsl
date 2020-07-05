#version 330 core
const int num_spheres = 80;
const float width = 1920;
const float height = 1080;
//const float width = 1280;
//const float height = 720;

uniform vec4 sp[(num_spheres+2)*2];
uniform sampler2D terrain;
in vec4 gl_FragCoord;
out vec4 fragColor;

// prenormalized to avoid having it in the script -> made it worse?!
// const vec3 sun_dir = vec3( 0.55, 0.61, 0.56 );
const float maximum_dist = 99999.0;

vec2 water[4] = vec2[4]( normalize(vec2( 0.23, 0.65  )), 
                        normalize(vec2( 0.83, -0.26  )),
                        normalize(vec2( 0.13, -0.83  )),
                        normalize(vec2( -0.2, 0.55  )));



vec3 water_ripple( vec3 pos ) {
    float intensity = 0.0;
    float intensity2 = 0.0;
    for( int k=0; k< 4; k++ ) {
        float t = pos.x*water[k].x + pos.z*water[k].y;
        t = t*(4.0-(float(k)*0.51013))+sp[162].z;

        float mt = 1.0/(float(k)+1.0);
        intensity += mt*sin(t-0.3*cos(t));
        intensity2 += mt*cos(t-0.3*sin(t));
    }
    return vec3( intensity, 0.0, intensity2 );
}

bool w_intersect_sphere( vec3 ray_dir, vec3 origin, vec3 sphere, float sphere_radius2, out float ott ) {
   // intersect with sphere 
    vec3 origToSphere = sphere - origin;
    float tCA = dot( origToSphere, ray_dir);
    if( tCA < 0.0 ) {
        // ray center is towards back of ray. cant intsesect
        return false;
    } else 
    {
        float dd = length(origToSphere);
        float distToMidpoint2 = dd*dd-tCA*tCA;
        if( distToMidpoint2 > sphere_radius2 ) {
            return false;
        } 
        else {
            float thc = sqrt(sphere_radius2-distToMidpoint2);
            ott = tCA - thc;
            return true;
        }
    }
}

float get_height( vec2 pos, out float type ) {
    vec4 col = texture( terrain, pos/512.0  );
    type = col.y;
    return col.x*60.0-12.1;
}

void intersect_box( vec3 origin, vec3 delta, out float near, out float far ) {
    vec3 t1 = (vec3( 0. ) - origin )/delta;
    vec3 t2 = (vec3( 512. ) - origin )/delta;
    near = max( min( t1.x, t2.x ), min( t1.z, t2.z ) );
    far = min( max( t1.x, t2.x ), max( t1.z, t2.z ) );
}

bool cast_ray( vec3 origin, vec3 delta, out float t, out vec3 col, out vec3 normal, out float refrac, out float type ) 
{
    float near_t, far_t;
    intersect_box( origin,delta, near_t, far_t );

    if( far_t < near_t  ) {
        return false;
    }

    float skip_t =  max( 0., near_t );
    origin = origin + skip_t*delta;

    // Setup stepper vars
    vec2 tmax, tdelta, grid_step;
    grid_step = sign( delta.xz );
    tdelta = 1.0 / delta.xz * grid_step;         // inverts the sign for negs, making it equivalent to abs
    tmax = tdelta * (max(grid_step,0) - fract(origin.xz)*grid_step);
    // Seems to be increasing the size of the final
    if( any( isinf( tmax ) || isinf( tdelta ) ) ) {
        return false;
    }

    vec2 ip = floor( origin.xz );
    float next_y = origin.y;
    float old_height = get_height( ip, type );
    refrac = 1.2;
    
    for( t=0.0;t<far_t-skip_t;) {
        vec2 or = vec2( float(tmax.x < tmax.y), float(tmax.x >= tmax.y));
        t = dot(tmax,or);
        float y = origin.y + delta.y * t;
        ip = ip + grid_step*or;
        tmax = tmax + tdelta*or; 

        // check exit height
        if( old_height > y ) {
            col = vec3( 0.2, 0.071, .01 ) + step( 27, old_height ) * vec3( 0.1, -0.06, 0 );
            normal = vec3( 0, 1.0, 0 );

            // work out precise t ( maybe precision errors when delta.y is near 0)
            t = ( old_height - origin.y ) / delta.y; 
//            t += skip_t;  // in all code we should always start inside the map so skip_t should never be required
            return true;
        }

        // check entry height to next pos
        old_height = get_height( ip, type);
        if( old_height > y ) {
            refrac =1.5;
            col = vec3( .2, .2, .2 ) + step( 40, old_height ) * vec3( 0., -.03, -.1 );

            if( type == 1.0 ) {
                float threshold = (0.6335-0.01)*60.0 - 12.1;
                if( y < threshold ) {
                    type = 0.0;
                }
            }
            //t += skip_t;
            normal = vec3( -grid_step.x*or.x, 0, -grid_step.y*or.y );
            return true;
        }
    }
    return false;
}


float fresnel( float n2, vec3 normal, vec3 incident )
{
    // Schlick aproximation
    float r0 = (1.0-n2) / (1.0+n2);     // r0 could be precalced. Fresnel is always with respect to air
    r0 *= r0;
    float cosX = -dot(normal, incident);
    float x = 1.0-cosX;
    float ret = r0+(1.0-r0)*x*x*x*x*x;
    return ret;
}

const vec3 absorption_coeff  = vec3( 0.000005, 0.000015, 0.00027 )*15.0;
const vec3 scattering_coeff = vec3( 0.00015, 0.00015, 0.00027 )*15.0;


vec3 extinction( float dist ) {
    return      exp( -dist*( absorption_coeff + scattering_coeff ) );
}

vec3 in_scatter( float dist, float cos_angle ) {
    float rayleigh_scatter = .0003 / 16.0*3.14159* ( 1.0 + cos_angle*cos_angle ); 
    vec3 rayleigh_coeff  = 1.0 / ( absorption_coeff + scattering_coeff ) * ( 1.0-exp( -dist*( scattering_coeff ) ) );

    float mie_g = 0.476;
    vec3 mie_scatter =  vec3( 0.0020, 0.0008, 0.0002 ) * ( 1.0 - mie_g )*( 1.0 - mie_g ) / ( 4.0 * 3.14159 * pow( ( 1.0 + mie_g*mie_g  - 2.0 * mie_g *cos_angle ), 1.5 ) ); 
    float mie_coeff = 20.0 / (  absorption_coeff.x + scattering_coeff.x ) * ( 1.0-exp( -dist*( scattering_coeff.x ) ) );
    return rayleigh_scatter*rayleigh_coeff+mie_scatter*mie_coeff;
 }


void main()
{
    vec3 sun_dir = normalize( vec3( 1.0, 1.10, 1.0 ));

    // calculate normalized screen pos with center at 0,0 extending width/height,1 
    vec2 screen_pos_2d = 2.0*(gl_FragCoord.xy/height) - vec2( width/height, 1.0 );

    // establish the 3d normalized 3d position, camera is at 0,0,0,   ray is towards screen_pos, depth
//    vec3 camera_tgt_3d = vec3( screen_pos_2d, -2.0 );
    //vec3 camera_pos_3d = vec3( 0., 0., 0.);       // no need to track as it is at 0,0,0
    vec4 co = cos( sp[ 161 ] );         // a= co.y   c = co.x
    vec4 si = sin( sp[ 161 ] );         // b= si.y   d = si.x
    mat3 rot_m = mat3(  co.y,      0,     -si.y,
                        -si.x*si.y,   co.x,      -si.x*co.y,
                        co.x*si.y,    si.x,    co.y*co.x );
    // no roll at the moment
//     float _angle3 = sp[ 161 ].z;
//     mat3 roll = mat3(  cos(_angle3),     -sin( _angle3) ,       0 , 
//                         sin(_angle3),     cos(_angle3),0 ,
//                         0,     0, 1 );
//    rot_m = rot_m * tilt_m * roll;
    
    // vec3 origin = rot_m*camera_pos_3d; no need to rotate origin, Its at 0,0,0
    vec3 dest = rot_m*vec3( screen_pos_2d, -2.0 );
//    vec3 dest = rot_m*camera_tgt_3d;

    vec3 origin = sp[160].xyz;      // camera at translated origin
    dest += origin;

    vec3 ray_dir = normalize( dest - origin );

    float contribution = 1.0;
    vec3 final_color = vec3( 0);

    for( int bounce =2; bounce >0 ; bounce -- ) {
        vec3 new_ray_dir;
        vec3 norm;
        vec3 pos;
        vec3 diffuseCol;
        float refractive_index;
        float reflectance = 0.0;
        float current_t = maximum_dist;

        for( int idx=0; idx < num_spheres; idx++ ) {
            float n_t;          // For some reason I cant pass current_t as out var into the func. Somehow the compiler seems to optimize out
                                // the preceeding assignment if I do
            if( w_intersect_sphere( ray_dir, origin, sp[idx*2].xyz, sp[idx*2].w, n_t) ) {
                if( n_t < current_t ) {
                    current_t = n_t;
                    pos = origin + current_t*ray_dir;
                    norm = normalize( pos-sp[idx*2].xyz);
                    diffuseCol = sp[idx*2+1].xyz;  // vec3( 0.02, .02, 0.02 );
                    refractive_index =  sp[idx*2+1].w;       // 1.3171;
                    reflectance = fresnel( refractive_index, norm, ray_dir);
                }
            }
        }
            
        //Check if we hit the sceneary
        float grid_t;
        vec3 diffuseCol2;
        vec3 norm2;
        float type;
        if( cast_ray(origin, ray_dir, grid_t, diffuseCol2, norm2, refractive_index, type ) ) {
            // hit the scenery, if it is closer than the sphere this overrides
            if( grid_t < current_t ) {
                current_t = grid_t;
                diffuseCol = diffuseCol2;
                norm = norm2;
                pos = origin + ray_dir * current_t*0.9999;
                reflectance = fresnel( refractive_index, norm, ray_dir);
            }
        } else if( ray_dir.y < 0.0 && current_t == maximum_dist) {
            current_t = ( -10.5-origin.y ) /  ray_dir.y;//   ground_plane_intersect( ray_dir, origin , -0.5 );
        }

        grid_t = ( -0.5-origin.y ) /  ray_dir.y;//   ground_plane_intersect( ray_dir, origin , -0.5 );
        if( ray_dir.y < 0.0 && grid_t <= current_t ) {
            pos = origin + ray_dir * grid_t*0.9999;
            // divide angle effect by distance to avoid grazing angle problems where the ripple would put the camera direction 'below' the water normal
            norm = vec3( 0.0, 1.0f, 0.0f ) + water_ripple( pos )*0.03/grid_t;     
            norm = normalize(norm);

            reflectance = fresnel( 1.1, norm, ray_dir);

            //bend and rethrow ray underwater
            vec3 uw_dir = refract( ray_dir, norm, 1.-reflectance);
            diffuseCol = vec3( 0.05, 0.05, 0.15 );                
            if( cast_ray(pos, uw_dir*100, grid_t, diffuseCol2, norm2, refractive_index, type ) ) {
                diffuseCol += diffuseCol2 * exp( -grid_t*40.0 );
            }
        } 
        new_ray_dir = reflect( normalize( ray_dir ), norm );

        if( current_t >= maximum_dist || bounce == 1 ) {
            final_color += in_scatter( current_t, dot( sun_dir,ray_dir) ) * contribution;
            break;
        }

        // // light the point
        // Is the light shadowed
        bool in_shade = cast_ray( pos, sun_dir, grid_t, diffuseCol2, norm2, refractive_index, type );
        if( !in_shade ) 
        {
            for( int idx=0; idx < num_spheres; idx++ ) 
            {
                if( w_intersect_sphere( sun_dir, pos, sp[idx*2].xyz, sp[idx*2].w, grid_t ) )  {
                    in_shade = true;
                    break;
                }
            }                
        }

        // reusing diffusecol2 for poitn col to avoid declaring extra var
        if( !in_shade)
        {
            float diffuse = dot( sun_dir, norm );
            vec3 halfway = normalize( sun_dir-ray_dir );        // halfwar between vectors pointing towards camera and sun
            float specular = pow( dot( norm, halfway ), 121.0 );
            specular = clamp( specular, 0.0, 1.0 );
            diffuseCol2 = vec3(specular) + diffuseCol * diffuse;
        } else {
            diffuseCol2 = diffuseCol* 0.02;
        }
        // attenuate
        diffuseCol2 *= extinction( current_t );
        diffuseCol2 += in_scatter( current_t, dot( sun_dir,ray_dir) );

        final_color += diffuseCol2 * contribution * ( 1.0 - reflectance );
        contribution = contribution * reflectance;
        ray_dir = new_ray_dir;
        origin = pos;

    }

    // vignetting
    // float cut_fraction = min( sp[162].x, sp[162].y );
    // if( cut_fraction <= 18 ) {
    //     cut_fraction = 1.-cut_fraction/8.;
    // } else {
    //     cut_fraction = 0.0;
    // }
    // cut_fraction = 0.0;
    // float dist = length( vec2( screen_pos_2d.x*(height/width), screen_pos_2d.y) );
    // float vignetting_level = min( 1.0, smoothstep( 0.95*(1.-cut_fraction/26.0), 1.31, dist )*0.6 + cut_fraction );

    // vec3 vfcolor = mix( final_color, vec3(0), vignetting_level );

    // //fragColor = vec4( pow( vfcolor, vec3(1.0 / 2.2) ), 1. );
    fragColor = vec4( pow( final_color, vec3(1.0 / 2.2) ), 1. );
}
