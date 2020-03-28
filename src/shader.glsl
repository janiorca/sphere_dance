#version 330 core
in vec4 gl_FragCoord;
out vec4 fragColor;
uniform float iTime;
uniform vec4 spheres[80*2];
const int num_spheres = 80;

const vec3 sun_dir = normalize( vec3( 1.0, 1.10, 1.0 ));
const float maximum_dist = 99999.0;

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

float ground_plane_intersect( vec3 ray_dir, vec3 origin, float ground, out vec3 pos, out vec3 norm ) {
    if( ray_dir.y >= 0.0 ) {
        return maximum_dist;
    }
    float t = ( ground-origin.y ) /  ray_dir.y; 
    norm = vec3( 0.0, 1.0f, 0.0f );
    pos = origin + ray_dir*t;
    return t;
}

float fresnel( float n, vec3 normal, vec3 incident )
{
    // Schlick aproximation
    float r0 = (1.0-n) / (1.0+n);
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
    vec3 burn = 4.0*vec3( 27.0, 4.0, 1.0 ) / pow( dist, 4.0 );
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


void main( )
{
    vec2 center = vec2( 0.5, 0.5 );
    float t = iTime*2.0;
    float adjustedTime = (30.0*t - 45.0*cos(t) + cos(3.0*t) - 9.0* sin(2.0*t))/96.0;
    adjustedTime += iTime*0.1;
    vec2 uv = gl_FragCoord.xy/1200.0;
    // [ -0.5, 05 ]    
    vec2 screen_pos_2d = 2.0*(uv - center);
    vec3 screen_pos_3d = vec3( screen_pos_2d, -39.0 );

    float _angle = adjustedTime/3.0 + iTime*0.2;
    mat3 rot_m = mat3( cos(_angle),0,  -sin( _angle ), 
                        0,          1,          0,
                        sin(_angle), 0, cos(_angle) );

    vec3 origin = rot_m*vec3( 0,0,-40.0) ;
    vec3 ray_dir = normalize(rot_m*screen_pos_3d - origin);

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

        current_t = ground_plane_intersect( ray_dir, origin , -18.0, pos, norm );
        if( current_t <= maximum_dist ) {
            refractive_index = 1.77;
            diffuseCol = vec3( 0.05, 0.05, 0.05 );
            if( ( ( int( pos.x/5.0) +int(pos.z/5.0) ) & 1 )== 1){
                float d = ( pos.x*pos.x + pos.z*pos.z );
                diffuseCol = mix( diffuseCol, vec3( 0.59, 0.6, 0.5 ),
                     smoothstep( 8000.0, 4000.0, d ) );
            } 
        } else {
            diffuseCol = vec3( 1.0, 1.0, 0.0);
        }

        for( int idx=0; idx < num_spheres; idx++ ) {
            vec3 vv = vec3( 0.0, 0.0, 0.0 )*float(idx);
            current_t  =  w_intersect_sphere( current_t, ray_dir, origin, spheres[idx*2].xyz, spheres[idx*2].w, idx, 
                pos, norm, final_idx);
        }
        // workout out material properties
        if( final_idx >= 0 ) {
            diffuseCol = spheres[final_idx*2+1].xyz;  // vec3( 0.02, .02, 0.02 );
            refractive_index =  spheres[final_idx*2+1].w;       // 1.3171;
            reflectance = fresnel( refractive_index, norm, ray_dir);
            new_ray_dir = reflect( ray_dir, norm );
        }

        vec3 point_color = vec3( 0, 0, 0 );

        if( current_t >= maximum_dist ) {
            point_color += in_scatter( current_t, dot( sun_dir,ray_dir) );
            final_color += point_color * contribution;
            break;
        }

        // light the point
        for( int lt=0; lt<1; lt++ ) 
        {
            // Is the light shadowed
            bool in_shade = false;
            for( int idx=0; idx < num_spheres; idx++ ) 
            {
                if( intersects_sphere( sun_dir, pos, spheres[idx*2].xyz, spheres[idx*2].w ) ) 
                {
                    in_shade = true;
                    break;
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
        if( final_idx !=0 ) {
            point_color += add_burn( current_t, dot( sun_dir,ray_dir) );

        }
        point_color += in_scatter( current_t, dot( sun_dir,ray_dir) );

        final_color += point_color * contribution * ( 1.0 - reflectance );
        contribution = contribution * reflectance;
        ray_dir = new_ray_dir;
        origin = pos;

        if( final_idx == -1 ){
            break;
        }
    }
    vec3 fragFinal = pow( final_color, vec3(1.0 / 2.2) );
    fragColor = vec4(fragFinal, 1.0);
}
