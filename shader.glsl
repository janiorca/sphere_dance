#version 330 core
#define steps 165.0
const int num_spheres = 80;
const float width = 1280;
const float height = 720;
uniform float iTime;
uniform vec4 spheres[(num_spheres+2)*2];
uniform sampler2D terrain;
in vec4 gl_FragCoord;
out vec4 fragColor;


const vec3 sun_dir = normalize( vec3( 1.0, 1.10, 1.0 ));
const float maximum_dist = 99999.0;

vec2 water[4] = vec2[4]( normalize(vec2( 0.23, 0.6556  )), 
                        normalize(vec2( 0.83, -0.26  )),
                        normalize(vec2( 0.13, -0.826  )),
                        normalize(vec2( -0.2, 0.55  )));



vec3 water_ripple( vec3 pos ) {
    float intensity = 0.0;
    float intensity2 = 0.0;
    for( int k=0; k< 4; k++ ) {
        float t = pos.x*water[k].x + pos.z*water[k].y;
        t = t*(4.0-(float(k)*0.51013))+iTime;

        float mt = 1.0/(float(k)+1.0);
        intensity += mt*sin(t-0.3*cos(t));
        intensity2 += mt*cos(t-0.3*sin(t));
    }
    return vec3( intensity, 0.0, intensity2 );
}

float ground_plane_intersect( vec3 ray_dir, vec3 origin, float ground, out vec3 pos, out vec3 norm ) {
    if( ray_dir.y >= 0.0 ) {
        return maximum_dist;
    }
    float t = ( ground-origin.y ) /  ray_dir.y; 
    norm = vec3( 0.0, 1.0f, 0.0f );
    pos = origin + ray_dir*t;
    return t;
}


float w_intersect_sphere( float max_t, vec3 ray_dir, vec3 origin, 
    vec3 sphere, float sphere_radius2, int idx_in, 
    out vec3 pos, out vec3 norm, out int idx ) {
   // intersect with sphere 
    vec3 origToSphere = sphere - origin;
    float tCA = dot( origToSphere, ray_dir);
    if( tCA < 0.0 ) {
        // ray center is towards back of ray. cant intsesect
        return max_t;
    } else 
    {
        float dd = length(origToSphere);
        float distToMidpoint2 = dd*dd-tCA*tCA;
        if( distToMidpoint2 > sphere_radius2 ) {
            return max_t;
        } 
        else {
            float thc = sqrt(sphere_radius2-distToMidpoint2);
            float t0 = tCA - thc;           // entry 
            if( t0 < max_t ) {
                pos = origin + t0*ray_dir;
                norm = normalize( pos-sphere);
                idx = idx_in;
                return t0;
            } else {
                return max_t;
            }
        }
    }
}

// For shadows we only care if there was intersection
bool intersects_sphere( vec3 ray_dir, vec3 origin, vec3 sphere, float sphere_radius2 ) {
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
            return true;
        }
    }
}

float get_height( vec2 pos, out float level, out float type ) {
    pos += vec2( 256.0, 256.0 );
    vec4 col = texture( terrain, pos/512.0  );
    level = col.x* 5.0;
    type = col.y;
    return col.x*60.0-12.1;
}

void prep_stepper( vec3 origin, vec3 dest, 
    out float step_x, out float tDeltaX, out float tMaxX, out vec3 xNormal, 
    out float step_z, out float tDeltaZ, out float tMaxZ, out vec3 zNormal  ) 
{
    vec3 delta = dest - origin;
    if( delta.x > 0.0 ) {
        step_x = 1.0;
        tDeltaX = 1.0 / delta.x;
        tMaxX = tDeltaX * (1.0 - fract(origin.x));
        xNormal = vec3( -1.0, 0, 0);
    } else {
        step_x = -1.0;
        tDeltaX = 1.0 / -delta.x;
        tMaxX = tDeltaX * fract(origin.x); 
        xNormal = vec3( 1.0, 0, 0);
    }
    if( delta.z > 0.0 ) {
        step_z = 1.0;
        tDeltaZ = 1.0 / delta.z;
        tMaxZ = tDeltaZ * (1.0 - fract(origin.z));
        zNormal = vec3( 0, 0, -1.0 );
    } else {
        step_z = -1.0;
        tDeltaZ = 1.0 / -delta.z;
        tMaxZ = tDeltaZ * fract(origin.z); 
        zNormal = vec3( 0, 0, 1.0 );
    }
}

bool cast_ray( vec3 origin, vec3 dest, float max_depth, out float t, out vec3 col, out vec3 normal, out float refrac, 
out float type ) 
{
    vec3 delta = dest - origin;
    float step_x, tDeltaX, tMaxX;
    vec3 xNormal;
    float step_z, tDeltaZ, tMaxZ;
    vec3 zNormal;

    prep_stepper( origin, dest, step_x, tDeltaX, tMaxX, xNormal, step_z, tDeltaZ, tMaxZ, zNormal  );

    float x = floor( origin.x );
    float z = floor( origin.z );
    float y = 0.0;
    float next_y = origin.y;
    float level;
    float old_height = get_height( vec2( x, z ), level, type );
    refrac = 0.0;
    for( float depth = 0.0; depth < max_depth; depth ++ ) {
        if(tMaxX < tMaxZ) { 
            t = tMaxX;
            y = origin.y + delta.y * tMaxX;
            tMaxX= tMaxX + tDeltaX; 
            x= x + step_x; 
            normal = xNormal;
        } else 
        { 
            t = tMaxZ;
            y = origin.y + delta.y * tMaxZ;
            tMaxZ= tMaxZ + tDeltaZ; 
            z= z + step_z; 
            normal = zNormal;
        } 
        // check exit height
        if( old_height > y ) {
            col = vec3( level/40., level/40., 1.0);
            if( level <= 2.5 ){
                col = vec3( 0.2, 0.071, .01 );
                refrac = 1.1;
            } else {
                col = vec3( 0.8, 0.01, 0.01 );
                refrac = 1.5;
            }
            normal = vec3( 0, 1.0, 0 );

            // work out precise t ( maybe precision errors when delta.y is near 0)
            t = ( old_height - origin.y ) / delta.y; 

            return true;
        }

        // check entry height to next pos
        old_height = get_height( vec2( x, z ), level , type);
        if( old_height > y ) {
            if( level >= 3.5 ){
                col = vec3( 0.2, 0.171, 0.01 );
                refrac = 1.5;
            } else {
                col = vec3( 0.2, 0.2, 0.2 );
                refrac =1.5;
            }

            if( type == 1.0 ) {
                float threshold = (0.6335-0.01)*60.0 - 12.1;
                if( y < threshold ) {
                    type = 0.0;
                }
            }
                            


            return true;
        }
    }
    return false;
}


float fresnel( float n2, vec3 normal, vec3 incident )
{
    // Schlick aproximation
    float r0 = (1.0-n2) / (1.0+n2);
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

vec3 add_burn( float dist, float cos_angle ) {
    vec3 burn = 4.0*vec3( 27.0, 4.0, 1.0 ) / pow( dist*10.0, 4.0 );
    return burn;
 }

vec3 in_scatter( float dist, float cos_angle ) {
    float rayleigh_scatter = .0003 / 16.0*3.14159* ( 1.0 + cos_angle*cos_angle ); 

    vec3 rayleigh_coeff =         vec3( 1.0 / ( absorption_coeff.x + scattering_coeff.x ) * ( 1.0-exp( -dist*( scattering_coeff.x ) ) ),
                                        1.0 / ( absorption_coeff.y + scattering_coeff.y ) * ( 1.0-exp( -dist*( scattering_coeff.y ) ) ),
                                        1.0 / ( absorption_coeff.z + scattering_coeff.z ) * ( 1.0-exp( -dist*( scattering_coeff.z ) ) ) );

    float mie_g = 0.476;
    vec3 mie_scatter =  vec3( 0.0020, 0.0008, 0.0002 ) * ( 1.0 - mie_g )*( 1.0 - mie_g ) / ( 4.0 * 3.14159 * pow( ( 1.0 + mie_g*mie_g  - 2.0 * mie_g *cos_angle ), 1.5 ) ); 
    float mie_coeff = 20.0 / (  absorption_coeff.x + scattering_coeff.x ) 
                            * ( 1.0-exp( -dist*( scattering_coeff.x ) ) );
    return rayleigh_scatter*rayleigh_coeff+mie_scatter*mie_coeff;
 }


void main()
{
    float t = iTime*2.0;
    float adjustedTime = (30.0*t - 45.0*cos(t) + cos(3.0*t) - 9.0* sin(2.0*t))/96.0;
    adjustedTime += iTime*0.1;
//    float _angle = adjustedTime/3.0 + iTime*0.2;
//    float _angle = iTime*0.2;
//    float _angle2 = iTime*0.001202-0.02;

    // calculate normalized screen pos with center at 0,0 extending width/height,1 

    vec2 screen_pos_2d = 2.0*(gl_FragCoord.xy/height) - vec2( width/height, 1.0 );
    // establish the 3d normalized 3d position, camera is at 0,0,0,   ray is towards screen_pos, depth
    vec3 camera_tgt_3d = vec3( screen_pos_2d, -2.0 );
    vec3 camera_pos_3d = vec3( 0., 0., 0.);
    float _angle = spheres[ 161 ].y;
    mat3 rot_m = mat3( cos(_angle),0,  -sin( _angle ), 
                         0,          1,          0,
                         sin(_angle), 0, cos(_angle) );

    float _angle2 = spheres[ 161 ].x;
    mat3 tilt_m = mat3(  1,     0,       0 , 
                        0,     cos(_angle2),-sin( _angle2 ),
                        0,     sin(_angle2), cos(_angle2) );

    float _angle3 = spheres[ 161 ].z;
    mat3 roll = mat3(  cos(_angle3),     -sin( _angle3) ,       0 , 
                        sin(_angle3),     cos(_angle3),0 ,
                        0,     0, 1 );


   rot_m = rot_m * tilt_m * roll;

//    vec3 camera_translation = vec3( 0., .8, 2.);
    vec3 camera_translation = spheres[160].xyz;
    
    vec3 origin = rot_m*camera_pos_3d;
    vec3 dest = rot_m*camera_tgt_3d;

    origin += camera_translation;
    dest += camera_translation;

    vec3 ray_dir = normalize( dest - origin );

    float contribution = 1.0;
    vec3 final_color = vec3( 0,0,0);

    for( int bounce =2; bounce >0 ; bounce -- ) {
        vec3 new_ray_dir;
        vec3 norm;
        vec3 pos;
        vec3 diffuseCol;
        float refractive_index;
        float reflectance = 0.0;
        int final_idx = -1;
        float current_t = maximum_dist;
        float burn_coeff = 0.0;

        diffuseCol = vec3( 1.0, 1.0, 0.0);

        //Harmonize hit flagging 
        for( int idx=0; idx < num_spheres; idx++ ) {
            current_t  =  w_intersect_sphere( current_t, ray_dir, origin, spheres[idx*2].xyz, spheres[idx*2].w, idx, pos, norm, final_idx);
        }
        if( final_idx != -1 ) {
            // hit a sphere. tentative data
            diffuseCol = spheres[final_idx*2+1].xyz;  // vec3( 0.02, .02, 0.02 );
            refractive_index =  spheres[final_idx*2+1].w;       // 1.3171;
            reflectance = fresnel( refractive_index, norm, ray_dir);
            new_ray_dir = reflect( ray_dir, norm );
        }
            
        //Check if we hit the sceneary
        vec3 current_dest = origin + ray_dir;//*5.0;
        float grid_t;
        vec3 diffuseCol2;
        vec3 norm2;
        float type;
        float refractive_index2;
        if( cast_ray(origin, current_dest, steps, grid_t, diffuseCol2, norm2, refractive_index2, type ) ) {
            // hit the scenery, if it is closer than the sphere this overrides
            if( grid_t < current_t ) {
                if( type == 1.0 ) {
                    // rethrow ray for holo
//                    vec3 new_origin = fract(origin)*256;
//                    vec3 new_dest = fract(current_dest)*256;  //y-offsret
//                    float threshold = (0.6335-0.01)*60.0 - 12.1;
//                    float threshold = (0.6335-0.01)*60.0-10.6;
//                    float threshold = (0.6335-0.01)*60.0-11.2;

                    float threshold = (0.6335-0.01)*60.0-(12.1);//iTime*0.5+5.0);

                    vec3 new_origin= origin + ray_dir * grid_t*0.825;
                    new_origin = (new_origin-vec3( 130.0, threshold, 191.0 ))*512.0;
                    vec3 new_dest = (current_dest-vec3( 130.0, threshold, 191.0 ))*512.0;
//                    new_origin = (vec3( 130.0, threshold, 191.0 )-new_origin)*256.0;
//                    vec3 new_dest = (vec3( 130.0, threshold, 191.0 )-current_dest)*256.0;

                    float mini_grid_t;
                    if( cast_ray(new_origin, new_dest, 1512, mini_grid_t, diffuseCol2, norm2, refractive_index2, type ) ) {
                        current_t = grid_t;// + mini_grid_t/256.0;
                        diffuseCol = diffuseCol2;
                        norm = norm2;
                        refractive_index = refractive_index2;
                        pos = origin + ray_dir * current_t*0.9999;
                        reflectance = fresnel( refractive_index, norm, ray_dir);
                        new_ray_dir = reflect( normalize( ray_dir ), norm );
                    }
                } else {
                    current_t = grid_t;
                    diffuseCol = diffuseCol2;
                    norm = norm2;
                    refractive_index = refractive_index2;
                    pos = origin + ray_dir * current_t*0.9999;
                    reflectance = fresnel( refractive_index, norm, ray_dir);
                    new_ray_dir = reflect( normalize( ray_dir ), norm );
                }
            }
        }

//         vec3 pos2;
//         float g_t = ground_plane_intersect( ray_dir, origin , -0.5, pos2, norm2 );
//         if( g_t <= current_t ) {
//             refractive_index = 1.1;

//             pos = origin + ray_dir * g_t*0.9999;

// //            norm = norm2 + water_ripple( pos )*0.005;
//             norm = norm2 + water_ripple( pos )*0.01;
//             norm = normalize(norm);

//             reflectance = fresnel( refractive_index, norm, ray_dir);
//             new_ray_dir = reflect( ray_dir, norm );
//             final_idx = 0;

//             //bend and rethrow ray underwater
//             vec3 uw_dir = refract( ray_dir, norm, 1.-reflectance);
//             float uw_t;
//             diffuseCol = vec3( 0.05, 0.05, 0.15 );                
//             if( cast_ray(pos, pos+uw_dir*100.0, 115, uw_t, diffuseCol2, norm2, refractive_index2, type ) ) {
//                 diffuseCol += diffuseCol2 * exp( -uw_t*40.0 );//*-0.25 ) ); 
// //                diffuseCol = vec3( 12.,12.,0);//diffuseCol2 * 0;//vec3( exp( uw_t*-0.25 ), exp( uw_t*-0.5 ), exp( uw_t*-0.75 ) ); 
//             // } else {
//             //     diffuseCol = vec3( 0.05, 0.05, 0.15 );                
//             }
//             final_idx = 0;
//          } 


         vec3 point_color = vec3( 0, 0, 0 );
//         if( current_t >= maximum_dist ) {
//             point_color += in_scatter( current_t, dot( sun_dir,ray_dir) );
//             final_color += point_color * contribution;
//             break;
//         }

        // // light the point
        for( int lt=0; lt<1; lt++ ) 
        {
            // Is the light shadowed
            bool in_shade = false;
            vec3 blocker_col;
            vec3 blocker_normal;
            float blocker_refractive_index;
            vec3 sun_pos = pos + sun_dir*5.0;
            if( cast_ray( pos, sun_pos, 20.0, t, blocker_col, blocker_normal, blocker_refractive_index, type ) ) 
            {
                in_shade = true;
            }
            if( !in_shade ) 
            {
                for( int idx=0; idx < num_spheres; idx++ ) 
                {
                    if( intersects_sphere( sun_dir, pos, spheres[idx*2].xyz, spheres[idx*2].w ) ) 
                    {
                        in_shade = true;
                        break;
                    }
                }                
            }

            if( !in_shade)
            {
                vec3 reflectedLight = reflect( -sun_dir, norm );
                vec3 toCamera = -ray_dir;
                float diffuse = dot( sun_dir, norm );

                vec3 halfway = normalize( toCamera + sun_dir );
                float specular = pow( dot( norm, halfway ), 121.0 );
            
                specular = clamp( specular, 0.0, 1.0 );

                vec3 fragDiffuse = diffuseCol * diffuse;
                point_color += vec3(specular,specular,specular) + fragDiffuse;
            } else {
                point_color += diffuseCol* 0.02;
            }
        }
        // attenuate
        point_color *= extinction( current_t );
        // if( final_idx !=0 ) {
        //     point_color += add_burn( current_t, dot( sun_dir,ray_dir) );

        // }
        point_color += in_scatter( current_t, dot( sun_dir,ray_dir) );

        final_color += point_color * contribution * ( 1.0 - reflectance );
        contribution = contribution * reflectance;
        ray_dir = new_ray_dir;
        origin = pos;

    }
    vec3 fragFinal = pow( final_color, vec3(1.0 / 2.2) );
    fragColor = vec4(fragFinal, 1.0);
}
