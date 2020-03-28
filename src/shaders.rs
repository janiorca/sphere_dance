pub static frag_shader_src : &'static str = "
#version 330 core
#define steps 165.0
const int num_spheres = 80;
const float width = 1280;
const float height = 720;
uniform float iTime;
uniform vec4 spheres[num_spheres*2];
uniform sampler2D terrain;
in vec4 gl_FragCoord;
out vec4 fragColor;


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

float get_height( vec2 pos, out float level ) {
    pos += vec2( 272.0, 252.0 );
    vec4 col = texture( terrain, pos/512.0  );
    level = col.x* 5.0;
    return col.x*60.0;
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

bool cast_ray( vec3 origin, vec3 dest, float max_depth, out float t, out vec3 col, out vec3 normal, out float refrac ) 
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
    float old_height = get_height( vec2( x, z ), level );
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
        old_height = get_height( vec2( x, z ), level );
        if( old_height > y ) {
            col = vec3( level, level, 1.0);
            if( level >= 3.5 ){
                col = vec3( 0.2, 0.171, 0.01 );
                refrac = 1.5;
            } else {
                col = vec3( 0.8, 0.01, 0.01 );
                refrac =1.5;
            }

            return true;
        }
    }
    return false;
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
    float _angle = adjustedTime/3.0 + iTime*0.2;
//    float _angle = iTime*0.202;

    // calculate normalized screen pos with center at 0,0 extending width/height,1 

    vec2 screen_pos_2d = 2.0*(gl_FragCoord.xy/height) - vec2( width/height, 1.0 );
    // establish the 3d normalized 3d position, camera is at 0,0,0,   ray is towards screen_pos, depth
    vec3 camera_tgt_3d = vec3( screen_pos_2d, -2.0 );
    vec3 camera_pos_3d = vec3( 0., 0., 0.);
    mat3 rot_m = mat3( cos(_angle),0,  -sin( _angle ), 
                        0,          1,          0,
                        sin(_angle), 0, cos(_angle) );
    vec3 camera_translation = vec3( 0., 52.1, 2.);
//    vec3 camera_translation = vec3( 0., .1 + t/10.0, 2.);

    camera_pos_3d += camera_translation;
    camera_tgt_3d += camera_translation;

    vec3 origin = rot_m*camera_pos_3d;
    vec3 dest = rot_m*camera_tgt_3d;
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
        float refractive_index2;
        if( cast_ray(origin, current_dest, steps, grid_t, diffuseCol2, norm2, refractive_index2 ) ) {
            // hit the scenery, if it is closer than the sphere this overrides
            if( grid_t < current_t ) {
                current_t = grid_t;
                diffuseCol = diffuseCol2;
                norm = norm2;
                refractive_index = refractive_index2;
                pos = origin + ray_dir * current_t*0.9999;
                reflectance = fresnel( refractive_index, norm, ray_dir);
                new_ray_dir = reflect( normalize( ray_dir ), norm );
            }
        }

        vec3 point_color = vec3( 0, 0, 0 );
        if( current_t >= maximum_dist ) {
            point_color += in_scatter( current_t, dot( sun_dir,ray_dir) );
            final_color += point_color * contribution;
            break;
        }

        // // light the point
        for( int lt=0; lt<1; lt++ ) 
        {
            // Is the light shadowed
            bool in_shade = false;
            vec3 blocker_col;
            vec3 blocker_normal;
            float blocker_refractive_index;
            vec3 sun_pos = pos + sun_dir*5.0;
            // if( cast_ray( pos, sun_pos, 20.0, t, blocker_col, blocker_normal, blocker_refractive_index ) ) 
            // {
            //     in_shade = true;
            // }
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
        if( final_idx !=0 ) {
            point_color += add_burn( current_t, dot( sun_dir,ray_dir) );

        }
        point_color += in_scatter( current_t, dot( sun_dir,ray_dir) );

        final_color += point_color * contribution * ( 1.0 - reflectance );
        contribution = contribution * reflectance;
        ray_dir = new_ray_dir;
        origin = pos;

    }
    vec3 fragFinal = pow( final_color, vec3(1.0 / 2.2) );
    fragColor = vec4(fragFinal, 1.0);
//    fragColor.rgb = texture(  terrain, screen_pos_2d  ).rgb;
}\0\0";


//pub static frag_shader_src : &'static str = "#version 330 core
//in vec4 gl_FragCoord;out vec4 f;uniform float e;uniform vec4 d[160];const int k=80;const vec3 i=normalize(vec3(1.,1.1,1.));const float z=99999.;float t(float z,vec3 v,vec3 e,vec3 i,float f,int d,out vec3 x,out vec3 o,out int y){vec3 w=i-e;float k=dot(w,v);if(k<0.)return z;else{float p=length(w),s=p*p-k*k;if(s>f)return z;else{float t=sqrt(f-s),c=k-t;if(c<z)return x=e+c*v,o=normalize(x-i),y=d,c;else return z;}}}bool t(vec3 v,vec3 e,vec3 z,float f){vec3 i=z-e;float k=dot(i,v);if(k<0.)return false;else{float p=length(i),o=p*p-k*k;if(o>f)return false;else return true;}}float t(vec3 i,vec3 v,float f,out vec3 e,out vec3 x){if(i.y>=0.)return z;float k=(f-v.y)/i.y;x=vec3(0.,1.f,0.f);e=v+i*k;return k;}float t(float i,vec3 v,vec3 f){float e=(1.-i)/(1.+i);e*=e;float z=-dot(v,f),k=1.-z,o=e+(1.-e)*k*k*k*k*k;return o;}const vec3 x=vec3(5e-06,1.5e-05,.00027)*15.,o=vec3(.00015,.00015,.00027)*15.;vec3 t(float e){return exp(-e*(x+o));}vec3 t(float i,float v){vec3 k=4.*vec3(27.,4.,1.)/pow(i,4.);return k;}vec3 p(float i,float v){float k=.0003/16.*3.14159*(1.+v*v);vec3 f=vec3(1./(x.x+o.x)*(1.-exp(-i*o.x)),1./(x.y+o.y)*(1.-exp(-i*o.y)),1./(x.z+o.z)*(1.-exp(-i*o.z)));float e=.476;vec3 z=vec3(.002,.0008,.0002)*(1.-e)*(1.-e)/(12.5664*pow(1.+e*e-2.*e*v,1.5));float p=20./(x.x+o.x)*(1.-exp(-i*o.x));return k*f+z*p;}void main(){vec2 x=vec2(.5,.5);float o=e*2.,c=(30.*o-45.*cos(o)+cos(3.*o)-9.*sin(2.*o))/96.;c+=e*.1;vec2 w=gl_FragCoord.xy/1200.,s=2.*(w-x);vec3 r=vec3(s,-39.);float y=c/3.+e*.2;mat3 n=mat3(cos(y),0,-sin(y),0,1,0,sin(y),0,cos(y));vec3 u=n*vec3(0,0,-40.),b=normalize(n*r-u);float m=1.;vec3 l=vec3(0,0,0);for(int a=2;a>0;a--){vec3 h,g,q,Z;float Y,X=0.;int W=-1;float V=z,U=0.;V=t(b,u,-18.,q,g);if(V<=z){Y=1.77;Z=vec3(.05,.05,.05);if((int(q.x/5.)+int(q.z/5.)&1)==1){float T=q.x*q.x+q.z*q.z;Z=mix(Z,vec3(.59,.6,.5),smoothstep(8000.,4000.,T));}}else Z=vec3(1.,1.,0.);for(int T=0;T<k;T++){vec3 S=vec3(0.,0.,0.)*float(T);V=t(V,b,u,d[T*2].xyz,d[T*2].w,T,q,g,W);}if(W>=0)Z=d[W*2+1].xyz,Y=d[W*2+1].w,X=t(Y,g,b),h=reflect(b,g);vec3 T=vec3(0,0,0);if(V>=z){T+=p(V,dot(i,b));l+=T*m;break;}for(int S=0;S<1;S++){bool R=false;for(int Q=0;Q<k;Q++){if(t(i,q,d[Q*2].xyz,d[Q*2].w)){R=true;break;}}if(!R){vec3 Q=reflect(-i,g),P=-b;float O=dot(i,g);vec3 N=normalize(P+i);float M=pow(dot(g,N),121.);M=clamp(M,0.,1.);vec3 L=Z*O;T+=vec3(M,M,M)+L;}else T+=Z*.02;}T*=t(V);if(W!=0)T+=t(V,dot(i,b));T+=p(V,dot(i,b));l+=T*m*(1.-X);m=m*X;b=h;u=q;if(W==-1){break;}}vec3 V=pow(l,vec3(1./2.2));f=vec4(V,1.);}\0\0";

// in vec4 gl_FragCoord;
// out vec4 fragColor;
// uniform float iTime;
// uniform vec4 spheres[80*2];
// const int num_spheres = 80;

// const vec3 sun_dir = normalize( vec3( 1.0, 1.10, 1.0 ));
// const float maximum_dist = 99999.0;

// float w_intersect_sphere( float max_t, vec3 ray_dir, vec3 origin, 
//     vec3 sphere, float sphere_radius2, int idx_in, 
//     out vec3 pos, out vec3 norm, out int idx ) {
//    // intersect with sphere 
//     vec3 origToSphere = sphere - origin;
//     float tCA = dot( origToSphere, ray_dir);
//     if( tCA < 0.0 ) {
//         // ray center is towards back of ray. cant intsesect
//         return max_t;
//     } else 
//     {
//         float dd = length(origToSphere);
//         float distToMidpoint2 = dd*dd-tCA*tCA;
//         if( distToMidpoint2 > sphere_radius2 ) {
//             return max_t;
//         } 
//         else {
//             float thc = sqrt(sphere_radius2-distToMidpoint2);
//             float t0 = tCA - thc;           // entry 
//             if( t0 < max_t ) {
//                 pos = origin + t0*ray_dir;
//                 norm = normalize( pos-sphere);
//                 idx = idx_in;
//                 return t0;
//             } else {
//                 return max_t;
//             }
//         }
//     }
// }

// // For shadows we only care if there was intersection
// bool intersects_sphere( vec3 ray_dir, vec3 origin, vec3 sphere, float sphere_radius2 ) {
//    // intersect with sphere 
//     vec3 origToSphere = sphere - origin;
//     float tCA = dot( origToSphere, ray_dir);
//     if( tCA < 0.0 ) {
//         // ray center is towards back of ray. cant intsesect
//         return false;
//     } else 
//     {
//         float dd = length(origToSphere);
//         float distToMidpoint2 = dd*dd-tCA*tCA;
//         if( distToMidpoint2 > sphere_radius2 ) {
//             return false;
//         } 
//         else {
//             return true;
//         }
//     }
// }

// float ground_plane_intersect( vec3 ray_dir, vec3 origin, float ground, out vec3 pos, out vec3 norm ) {
//     if( ray_dir.y >= 0.0 ) {
//         return maximum_dist;
//     }
//     float t = ( ground-origin.y ) /  ray_dir.y; 
//     norm = vec3( 0.0, 1.0f, 0.0f );
//     pos = origin + ray_dir*t;
//     return t;
// }

// float fresnel( float n, vec3 normal, vec3 incident )
// {
//     // Schlick aproximation
//     float r0 = (1.0-n) / (1.0+n);
//     r0 *= r0;
//     float cosX = -dot(normal, incident);
//     float x = 1.0-cosX;
//     float ret = r0+(1.0-r0)*x*x*x*x*x;
//     return ret;
// }

// const vec3 absorption_coeff  = vec3( 0.000005, 0.000015, 0.00027 )*15.0;
// const vec3 scattering_coeff = vec3( 0.00015, 0.00015, 0.00027 )*15.0;

// vec3 extinction( float dist ) {
//     return      exp( -dist*( absorption_coeff + scattering_coeff ) );
// }

// vec3 add_burn( float dist, float cos_angle ) {
//     vec3 burn = 4.0*vec3( 27.0, 4.0, 1.0 ) / pow( dist, 4.0 );
//     return burn;
//  }

// vec3 in_scatter( float dist, float cos_angle ) {
//     float rayleigh_scatter = .0003 / 16.0*3.14159* ( 1.0 + cos_angle*cos_angle ); 

//     vec3 rayleigh_coeff =         vec3( 1.0 / ( absorption_coeff.x + scattering_coeff.x ) * ( 1.0-exp( -dist*( scattering_coeff.x ) ) ),
//                                         1.0 / ( absorption_coeff.y + scattering_coeff.y ) * ( 1.0-exp( -dist*( scattering_coeff.y ) ) ),
//                                         1.0 / ( absorption_coeff.z + scattering_coeff.z ) * ( 1.0-exp( -dist*( scattering_coeff.z ) ) ) );

//     float mie_g = 0.476;
//     vec3 mie_scatter =  vec3( 0.0020, 0.0008, 0.0002 ) * ( 1.0 - mie_g )*( 1.0 - mie_g ) / ( 4.0 * 3.14159 * pow( ( 1.0 + mie_g*mie_g  - 2.0 * mie_g *cos_angle ), 1.5 ) ); 
//     float mie_coeff = 20.0 / (  absorption_coeff.x + scattering_coeff.x ) 
//                             * ( 1.0-exp( -dist*( scattering_coeff.x ) ) );
//     return rayleigh_scatter*rayleigh_coeff+mie_scatter*mie_coeff;
//  }


// void main( )
// {
//     vec2 center = vec2( 0.5, 0.5 );
//     float t = iTime*2.0;
//     float adjustedTime = (30.0*t - 45.0*cos(t) + cos(3.0*t) - 9.0* sin(2.0*t))/96.0;
//     adjustedTime += iTime*0.1;
//     vec2 uv = gl_FragCoord.xy/1200.0;
//     // [ -0.5, 05 ]    
//     vec2 screen_pos_2d = 2.0*(uv - center);
//     vec3 screen_pos_3d = vec3( screen_pos_2d, -39.0 );

//     float _angle = adjustedTime/3.0 + iTime*0.2;
//     mat3 rot_m = mat3( cos(_angle),0,  -sin( _angle ), 
//                         0,          1,          0,
//                         sin(_angle), 0, cos(_angle) );

//     vec3 origin = rot_m*vec3( 0,0,-40.0) ;
//     vec3 ray_dir = normalize(rot_m*screen_pos_3d - origin);

//     float contribution = 1.0;
//     vec3 final_color = vec3( 0,0,0);

//     for( int bounce =2; bounce >0 ; bounce -- ) {
//         vec3 new_ray_dir;
//         vec3 norm;
//         vec3 pos;
//         vec3 diffuseCol;
//         float refractive_index;
//         float reflectance = 0.0;
//         int final_idx = -1;
//         float current_t = maximum_dist;
//         float burn_coeff = 0.0;

//         current_t = ground_plane_intersect( ray_dir, origin , -18.0, pos, norm );
//         if( current_t <= maximum_dist ) {
//             refractive_index = 1.77;
//             diffuseCol = vec3( 0.05, 0.05, 0.05 );
//             if( ( ( int( pos.x/5.0) +int(pos.z/5.0) ) & 1 )== 1){
//                 float d = ( pos.x*pos.x + pos.z*pos.z );
//                 diffuseCol = mix( diffuseCol, vec3( 0.59, 0.6, 0.5 ),
//                      smoothstep( 8000.0, 4000.0, d ) );
//             } 
//         } else {
//             diffuseCol = vec3( 1.0, 1.0, 0.0);
//         }

//         for( int idx=0; idx < num_spheres; idx++ ) {
//             vec3 vv = vec3( 0.0, 0.0, 0.0 )*float(idx);
//             current_t  =  w_intersect_sphere( current_t, ray_dir, origin, spheres[idx*2].xyz, spheres[idx*2].w, idx, 
//                 pos, norm, final_idx);
//         }
//         // workout out material properties
//         if( final_idx >= 0 ) {
//             diffuseCol = spheres[final_idx*2+1].xyz;  // vec3( 0.02, .02, 0.02 );
//             refractive_index =  spheres[final_idx*2+1].w;       // 1.3171;
//             reflectance = fresnel( refractive_index, norm, ray_dir);
//             new_ray_dir = reflect( ray_dir, norm );
//         }

//         vec3 point_color = vec3( 0, 0, 0 );

//         if( current_t >= maximum_dist ) {
//             point_color += in_scatter( current_t, dot( sun_dir,ray_dir) );
//             final_color += point_color * contribution;
//             break;
//         }

//         // light the point
//         for( int lt=0; lt<1; lt++ ) 
//         {
//             // Is the light shadowed
//             bool in_shade = false;
//             for( int idx=0; idx < num_spheres; idx++ ) 
//             {
//                 if( intersects_sphere( sun_dir, pos, spheres[idx*2].xyz, spheres[idx*2].w ) ) 
//                 {
//                     in_shade = true;
//                     break;
//                 }
//             }
//             if( !in_shade)
//             {
//                 vec3 reflectedLight = reflect( -sun_dir, norm );
//                 vec3 toCamera = -ray_dir;
//                 float diffuse = dot( sun_dir, norm );

//                 vec3 halfway = normalize( toCamera + sun_dir );
//                 float specular = pow( dot( norm, halfway ), 121.0 );
            
//                 specular = clamp( specular, 0.0, 1.0 );

//                 vec3 fragDiffuse = diffuseCol * diffuse;
//                 point_color += vec3(specular,specular,specular) + fragDiffuse;
//             } else {
//                 point_color += diffuseCol* 0.02;
//             }
//         }
//         // attenuate
//         point_color *= extinction( current_t );
//         if( final_idx !=0 ) {
//             point_color += add_burn( current_t, dot( sun_dir,ray_dir) );

//         }
//         point_color += in_scatter( current_t, dot( sun_dir,ray_dir) );

//         final_color += point_color * contribution * ( 1.0 - reflectance );
//         contribution = contribution * reflectance;
//         ray_dir = new_ray_dir;
//         origin = pos;

//         if( final_idx == -1 ){
//             break;
//         }
//     }
//     vec3 fragFinal = pow( final_color, vec3(1.0 / 2.2) );
//     fragColor = vec4(fragFinal, 1.0);
// }\0\0";
