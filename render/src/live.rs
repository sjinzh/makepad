use crate::cx::*;
use makepad_shader_compiler::shaderast::DrawShaderNodePtr;

#[derive(Clone, Debug)]
pub struct LiveBody {
    pub file: String,
    pub module_path: String,
    pub line: usize,
    pub column: usize,
    pub code: String,
    pub live_types: Vec<LiveType>
}

impl Cx {
    pub fn live_register(&mut self) {
        crate::DrawQuad::live_register(self);
        crate::GeometryQuad2D::live_register(self);
        crate::shader_std::define_shader_stdlib(self);
    }
    
    // ok so now what. now we should run the expansion
    pub fn live_expand(&mut self) {
        // lets expand the f'er
        let mut errs = Vec::new();
        self.shader_registry.live_registry.expand_all_documents(&mut errs);
        for err in errs {
            println!("Error expanding live file {}", err);
        }
    }
    
    pub fn verify_type_signature(&self, live_ptr: LivePtr, live_type: LiveType) -> bool {
        let node = self.shader_registry.live_registry.resolve_ptr(live_ptr);
        if let LiveValue::LiveType(ty) = node.value {
            if ty == live_type {
                return true
            }
        }
        false
    }
    
    pub fn register_live_body(&mut self, live_body: LiveBody) {
        // ok so now what.
        //println!("{}", live_body.code);
        //let cm = CrateModule::from_module_path_check(&live_body.module_path).unwrap();
        //println!("register_live_body: {}", ModulePath::from_str(&live_body.module_path).unwrap());
        let result = self.shader_registry.live_registry.parse_live_file(
            &live_body.file,
            ModulePath::from_str(&live_body.module_path).unwrap(),
            live_body.code,
            live_body.live_types
        );
        if let Err(msg) = result {
            println!("Error parsing live file {}", msg);
        }
    }
    
    pub fn register_factory(&mut self, live_type: LiveType, factory: Box<dyn LiveFactory>) {
        self.live_factories.insert(live_type, factory);
    }
    
    pub fn get_factory(&mut self, live_type: LiveType) -> &Box<dyn LiveFactory> {
        self.live_factories.get(&live_type).unwrap()
    }
    
    pub fn get_shader_from_ptr(&mut self, live_ptr: LivePtr, geometry_fields: &dyn GeometryFields) -> Option<Shader> {
        // lets first fetch the shader from live_ptr
        // if it doesn't exist, we should allocate and
        if let Some(shader_id) = self.live_ptr_to_shader_id.get(&live_ptr) {
            Some(Shader{
                live_ptr,
                shader_id:*shader_id
            })
        }
        else {
            fn live_type_to_shader_ty(live_type: LiveType) -> Option<Ty> {
                if live_type == f32::live_type() {Some(Ty::Float)}
                else if live_type == Vec2::live_type() {Some(Ty::Vec2)}
                else {None}
            }
            // ok ! we have to compile it
            let live_factories = &self.live_factories;
            let result = self.shader_registry.analyse_draw_shader(DrawShaderNodePtr(live_ptr), | span, id, live_type, draw_shader_def | {
                if id == id!(rust_type) {
                    if let Some(lf) = live_factories.get(&live_type) {
                        let mut fields = Vec::new();
                        lf.live_fields(&mut fields);
                        
                        let mut is_instance = false;
                        for field in fields {
                            if field.id == id!(geometry) {
                                is_instance = true;
                                continue
                            }
                            
                            if let Some(ty) = live_type_to_shader_ty(field.live_type) {
                                if is_instance {
                                    draw_shader_def.add_instance(field.id, ty, span);
                                }
                                else {
                                    draw_shader_def.add_uniform(field.id, ty, span);
                                }
                            };
                        }
                    }
                }
                if id == id!(geometry) {
                    if let Some(lf) = live_factories.get(&live_type) {
                        if lf.live_type() == geometry_fields.live_type_check() {
                            let mut fields = Vec::new();
                            geometry_fields.geometry_fields(&mut fields);
                            for field in fields {
                                draw_shader_def.add_geometry(field.id, field.ty, span);
                            }
                        }
                        else {
                            eprintln!("lf.get_type() != geometry_fields.live_type_check()");
                        }
                    }
                }
            });
            // ok lets print an error
            match result {
                Err(e) => {
                    println!("Error {}", e.to_live_file_error("", ""));
                }
                Ok(draw_shader_def) => {
                    // OK! SO the shader parsed
                    let shader_id = self.shaders.len();
                    let mut mapping =  CxShaderMapping::from_draw_shader_def(draw_shader_def, true);
                    mapping.update_live_uniforms(&self.shader_registry.live_registry);
                    
                    self.shaders.push(CxShader {
                        name: "todo".to_string(),
                        default_geometry: geometry_fields.get_geometry(),
                        platform: None,
                        mapping: mapping
                    });
                    // ok so. maybe we should fill the live_uniforms buffer?
                    
                    self.live_ptr_to_shader_id.insert(live_ptr, shader_id);
                    self.shader_compile_set.insert(live_ptr);
                    // now we simply queue it somewhere somehow to compile.
                    return Some(Shader {
                        shader_id,
                        live_ptr
                    });
                    // also we should allocate it a Shader object
                }
            }
            None
        }
        
        
    }
}

pub trait LiveFactory {
    fn live_new(&self, cx: &mut Cx) -> Box<dyn LiveUpdate>;
    fn live_fields(&self, fields: &mut Vec<LiveField>);
    fn live_type(&self) -> LiveType;
}

pub trait LiveNew {
    fn live_new(cx: &mut Cx) -> Self;
    fn live_type() -> LiveType;
    fn live_register(cx: &mut Cx);
}

pub trait LiveUpdate {
    fn live_update(&mut self, cx: &mut Cx, ptr: LivePtr);
    fn _live_type(&self) -> LiveType;
}

#[derive(Default)]
pub struct LiveBinding {
    pub live_ptr: Option<LivePtr>
}



#[macro_export]
macro_rules!live_prim {
    ( $ ty: ident, $ update: expr) => {
        impl LiveUpdate for $ ty {
            fn live_update(&mut self, _cx: &mut Cx, _ptr: LivePtr) {
                $ update;
            }
            
            fn _live_type(&self) -> LiveType {
                Self::live_type()
            }
        }
        impl LiveNew for $ ty {
            fn live_new(_cx: &mut Cx) -> Self {
                $ ty::default()
            }
            fn live_type() -> LiveType {
                LiveType(std::any::TypeId::of::< $ ty>())
            }
            fn live_register(cx: &mut Cx) {
                struct Factory();
                impl LiveFactory for Factory {
                    fn live_new(&self, cx: &mut Cx) -> Box<dyn LiveUpdate> where Self: Sized {
                        Box::new( $ ty ::live_new(cx))
                    }
                    
                    fn live_fields(&self, _fields: &mut Vec<LiveField>) where Self: Sized {
                    }
                    
                    fn live_type(&self) -> LiveType where Self: Sized {
                        $ ty::live_type()
                    }
                }
                cx.live_factories.insert( $ ty::live_type(), Box::new(Factory()));
            }
        }
    }
}

live_prim!(f32, {});
live_prim!(Vec2, {});

#[derive(Debug)]
pub struct LiveField {
    pub id: Id,
    pub live_type: LiveType
}


/*
pub trait DrawInputType {
    fn slots() -> usize;
    fn to_ty() -> Ty;
    // this writes a value to the area (wether texture, uniform or instance)
    fn write_draw_input(self, cx: &mut Cx, area: Area, id: Id, name: &str);
    /*
    fn last_animate(animator:&Animator, live_item_id: LiveItemId)->Option<Self> where Self: Sized;
    fn animate(cx: &mut Cx, animator:&mut Animator, time:f64, live_item_id: LiveItemId)->Option<Self> where Self: Sized;
    */
}

impl DrawInputType for f32 {
    fn slots() -> usize {1}
    
    fn to_ty() -> Ty {
        Ty::Float
    }
    
    fn write_draw_input(self, cx: &mut Cx, area: Area, id: Id, name: &str) {
        if let Some(wr) = area.get_write_ref(cx, id, Ty::Float, name) {
            for i in 0..wr.repeat {
                wr.buffer[i * wr.stride] = self;
            }
        }
    }
/*
    fn last_animate(animator:&Animator, live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        animator.last_float(live_item_id)
    }
    
    fn animate(cx: &mut Cx, animator:&mut Animator, time:f64, live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        animator.calc_float(cx, live_item_id, time)
    }*/
}

impl DrawInputType for Vec2 {
    fn slots() -> usize {2}
    
    fn to_ty() -> Ty {
        Ty::Vec2
    }
    
    fn write_draw_input(self, cx: &mut Cx, area: Area, id: Id, name: &str) {
        if let Some(wr) = area.get_write_ref(cx, id, Ty::Vec2, name) {
            for i in 0..wr.repeat {
                wr.buffer[i * wr.stride + 0] = self.x;
                wr.buffer[i * wr.stride + 1] = self.y;
            }
        }
    }
/*
    fn last_animate(animator:&Animator, live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        animator.last_vec2(live_item_id)
    }
    
    fn animate(cx: &mut Cx, animator:&mut Animator, time:f64, live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        animator.calc_vec2(cx, live_item_id, time)
    }*/
}

impl DrawInputType for Vec3 {
    fn slots() -> usize {3}
    
    fn to_ty() -> Ty {
        Ty::Vec3
    }
    
    fn write_draw_input(self, cx: &mut Cx, area: Area, id: Id, name: &str) {
        if let Some(wr) = area.get_write_ref(cx, id, Ty::Vec3, name) {
            for i in 0..wr.repeat {
                wr.buffer[i * wr.stride + 0] = self.x;
                wr.buffer[i * wr.stride + 1] = self.y;
                wr.buffer[i * wr.stride + 2] = self.z;
            }
        }
    }

/*
    fn last_animate( animator:&Animator, live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        animator.last_vec3(live_item_id)
    }
    
    fn animate(cx: &mut Cx, animator:&mut Animator, time:f64, live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        animator.calc_vec3(cx, live_item_id, time)
    }*/

}

impl DrawInputType for Vec4 {
    fn slots() -> usize {4}
    
    fn to_ty() -> Ty {
        Ty::Vec4
    }
    
    fn write_draw_input(self, cx: &mut Cx, area: Area, id: Id, name: &str) {
        if let Some(wr) = area.get_write_ref(cx, id, Ty::Vec4, name) {
            for i in 0..wr.repeat {
                wr.buffer[i * wr.stride + 0] = self.x;
                wr.buffer[i * wr.stride + 1] = self.y;
                wr.buffer[i * wr.stride + 2] = self.z;
                wr.buffer[i * wr.stride + 3] = self.w;
            }
        }
    }

/*
    fn last_animate(animator:&Animator, live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        animator.last_vec4(live_item_id)
    }
    
    fn animate(cx: &mut Cx, animator:&mut Animator, time:f64, live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        animator.calc_vec4(cx, live_item_id, time)
    }*/

}

impl DrawInputType for Mat4 {
    fn slots() -> usize {16}
    
    fn to_ty() -> Ty {
        Ty::Mat4
    }
    
    // find uniform, then find instance prop
    fn write_draw_input(self, cx: &mut Cx, area: Area, id: Id, name: &str) {
        if let Some(wr) = area.get_write_ref(cx, id, Ty::Mat4, name) {
            for i in 0..wr.repeat {
                for j in 0..16 {
                    wr.buffer[i * wr.stride + j] = self.v[j];
                }
            }
        }
    }
/*
    fn last_animate(_animator:&Animator, _live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        None
    }
    
    fn animate(_cx: &mut Cx, _animator:&mut Animator, _time:f64, _live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        None
    }*/

}

impl DrawInputType for Texture2D {
    fn slots() -> usize {0}
    
    fn to_ty() -> Ty {
        Ty::Texture2D
    }
    
    fn write_draw_input(self, cx: &mut Cx, area: Area, id: Id, name: &str) {
        if let Some(u) = self.0 {
            area.write_texture_2d_id(cx, id, name, u as usize)
        }
    }
/*
    fn last_animate(_animator:&Animator, _live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        None
    }
    
    fn animate(_cx: &mut Cx, _animator:&mut Animator, _time:f64, _live_item_id: LiveItemId)->Option<Self> where Self: Sized{
        None
    }*/
}

#[macro_export]
macro_rules!write_draw_input {
    ( $ cx: ident, $ area: expr, $ path: path, $ value: expr) => {
        ( $ value).write_draw_input( $ cx, $ area, live_str_to_id(module_path!(), stringify!( $ path)), stringify!( $ path))
    }
}



#[macro_export]
macro_rules!default_shader {
    () => {
        Shader {shader_id: 0, location_hash: live_location_hash(file!(), line!() as u64, column!() as u64)}
    }
}

/*
#[macro_export]
macro_rules!default_shader_overload {
    ( $ cx: ident, $ base: ident, $ path: path) => {
        $ cx.live_styles.get_default_shader_overload(
            $ base,
            live_str_to_id(module_path!(), stringify!( $ path)),
            module_path!(),
            stringify!( $ path)
        )
    }
}

#[macro_export]
macro_rules!default_shader {
    () => {
        Shader {shader_id: 0, location_hash: live_location_hash(file!(), line!() as u64, column!() as u64)}
    }
}*/*/

#[macro_export]
macro_rules!uid {
    () => {{
        struct Unique {}
        std::any::TypeId::of::<Unique>().into()
    }};
}