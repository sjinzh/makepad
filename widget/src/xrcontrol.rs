
use makepad_render::*;
use crate::widgetstyle::*;

#[derive(Clone)]
pub struct XRControl {
    pub cursor_view: View,
    pub ray_view: View,
    pub ray_cube: Cube,
    pub ray_cursor: Quad,
    pub animator: Animator,
    pub last_xr_update: Option<XRUpdateEvent>,
    
    pub _left_ray_area: Area,
    pub _right_ray_area: Area,
    pub _left_cursor_area: Area,
    pub _right_cursor_area: Area,
}

pub enum XRControlEvent {
    None
}

impl XRControl {
    pub fn new(cx: &mut Cx) -> Self {
        Self {
            ray_view: View::new(cx),
            cursor_view: View::new(cx),
            ray_cube: Cube::new(cx),
            ray_cursor: Quad {
                z: 3.0,
                ..Quad::new(cx)
            },
            animator: Animator::default(),
            last_xr_update: None,
            
            _left_ray_area: Area::Empty,
            _right_ray_area: Area::Empty,
            _left_cursor_area: Area::Empty,
            _right_cursor_area: Area::Empty
        }
    }
    
    pub fn shader_ray_cube() -> ShaderId {uid!()}
    pub fn shader_ray_cursor() -> ShaderId {uid!()}
    
    pub fn style(cx: &mut Cx, _opt: &StyleOptions) {
        // lets define the shader
        Self::shader_ray_cube().set(cx, Cube::def_cube_shader().compose(shader!{"
            
        "}));
        
        Self::shader_ray_cursor().set(cx, Quad::def_quad_shader().compose(shader!{"
            fn pixel() -> vec4 {
                let df = Df::viewport(pos * vec2(w, h));
                df.circle(0.5 * w, 0.5 * h, 0.5 * w);
                return df.fill(pick!(white));
            }
        "}));
        
    }
    
    pub fn handle_xr_control(&mut self, cx: &mut Cx, xr_event: &XRUpdateEvent, window_view: &View) -> Vec<Event> {
        
        let view_rect = window_view.get_rect(cx);
        
        let window_mat = Mat4::rotate_tsrt(
            Vec3 {x: 0., y: -view_rect.h, z: 0.0},
            -0.0005,
            Vec3 {x: -0.0, y: -180.0, z: 0.0},
            Vec3 {x: -0.20, y: -0.15, z: -0.3},
        );
        
        window_view.set_view_transform(cx, &window_mat);
        
        // lets set the left_input matrix
        let left_ray_matrix = Mat4::from_transform(xr_event.left_input.ray); // Mat4::from_mul(&Mat4::rotate(45.0, 0.0, 0.0), &Mat4::from_transform(xr_event.left_input.grip));
        let right_ray_matrix = Mat4::from_transform(xr_event.right_input.ray); //Mat4::from_mul(&Mat4::rotate(45.0, 0.0, 0.0), &Mat4::from_transform(xr_event.right_input.grip));
        
        self.ray_view.set_view_transform(cx, &Mat4::identity());
        self.last_xr_update = Some(xr_event.clone());
        
        self._left_ray_area.write_mat4(cx, Cube::transform(), &left_ray_matrix);
        self._right_ray_area.write_mat4(cx, Cube::transform(), &right_ray_matrix);
        
        // we have 2 points, 0,0,0 and 0,0,1? pointing straight back
        // then, we transform those with our left input ray
        let inv_window_mat = window_mat.invert();
        let right_origin = inv_window_mat.transform_vec4(right_ray_matrix.transform_vec4(Vec4 {x: 0., y: 0., z: 0., w: 1.0}));
        let right_vector = inv_window_mat.transform_vec4(right_ray_matrix.transform_vec4(Vec4 {x: 0., y: 0., z: 1., w: 1.0}));
        let left_origin = inv_window_mat.transform_vec4(left_ray_matrix.transform_vec4(Vec4 {x: 0., y: 0., z: 0., w: 1.0}));
        let left_vector = inv_window_mat.transform_vec4(left_ray_matrix.transform_vec4(Vec4 {x: 0., y: 0., z: 1., w: 1.0}));
        // now we have 2 points that make a line
        // we now simply need to intersect with the plane view_rect.w, view_rect.h, 0.
        let window_plane = Plane::from_points(
            Vec3 {x: 0., y: 0., z: 0.},
            Vec3 {x: view_rect.w, y: 0., z: 0.},
            Vec3 {x: 0., y: view_rect.h, z: 0.}
        );
        let right_pt = window_plane.intersect_line(right_origin.to_vec3(), right_vector.to_vec3()).to_vec2();
        let left_pt = window_plane.intersect_line(left_origin.to_vec3(), left_vector.to_vec3()).to_vec2();
        
        self._right_cursor_area.write_float(cx, Quad::x(), right_pt.x - 5.0);
        self._right_cursor_area.write_float(cx, Quad::y(), right_pt.y - 5.0);
        self._left_cursor_area.write_float(cx, Quad::x(), left_pt.x - 5.0);
        self._left_cursor_area.write_float(cx, Quad::y(), left_pt.y - 5.0);
        
        let mut events = Vec::new();
        
        fn do_input_event(cx:&mut Cx, digit:usize, pt:Vec2, time:f64, input:&XRInput, last_input:&XRInput)->Event{
             if input.buttons[0].pressed != last_input.buttons[0].pressed{
                // we have finger up or down
                if input.buttons[0].pressed{
                    return Event::FingerDown(FingerDownEvent {
                        digit:digit,
                        window_id: 0,
                        tap_count: 0,
                        abs: pt,
                        rel: pt,
                        handled: false,
                        is_touch: true,
                        rect: Rect::default(),
                        modifiers: KeyModifiers::default(),
                        time: time
                    });
                }
                else{
                    return Event::FingerUp(FingerUpEvent {
                        digit:digit,
                        window_id: 0,
                        abs: pt,
                        rel: pt,
                        is_over: false,
                        is_touch: true,
                        rect: Rect::default(),
                        abs_start: Vec2::default(),
                        rel_start: Vec2::default(),
                        modifiers: KeyModifiers::default(),
                        time: time
                    });
                }
                
            }
            else if input.buttons[0].pressed{ // we have move
                 return Event::FingerMove(FingerMoveEvent {
                    digit:digit,
                    window_id: 0,
                    abs: pt,
                    rel: pt,
                    rect: Rect::default(),
                    abs_start: Vec2::default(),
                    rel_start: Vec2::default(),
                    is_over: false,
                    is_touch: true,
                    modifiers: KeyModifiers::default(),
                    time: time
                });
            }
            cx.fingers[digit].over_last = Area::Empty;
            return Event::FingerHover(FingerHoverEvent {
                digit:digit,
                any_down: false,
                window_id: 0,
                abs: pt,
                rel: pt,
                rect: Rect::default(),
                handled: false,
                hover_state: HoverState::Over,
                modifiers: KeyModifiers::default(),
                time: time
            });
        }
        
        events.push(do_input_event(cx, 0, left_pt, xr_event.time, &xr_event.left_input, &xr_event.last_left_input));
        events.push(do_input_event(cx, 1, right_pt, xr_event.time, &xr_event.right_input, &xr_event.last_right_input));
       
        events
    }
    
    pub fn draw_xr_control(&mut self, cx: &mut Cx) {
        self.ray_cube.shader = Self::shader_ray_cube().get(cx);
        self.ray_cursor.shader = Self::shader_ray_cursor().get(cx);
        
        if self.cursor_view.begin_view(cx, Layout::abs_origin_zero()).is_ok() {
            self._left_cursor_area = self.ray_cursor.draw_quad_rel(cx, Rect {x: 0.0, y: 0.0, w: 10.0, h: 10.0}).into();
            self._right_cursor_area = self.ray_cursor.draw_quad_rel(cx, Rect {x: 0.0, y: 0.0, w: 10.0, h: 10.0}).into();
            self.cursor_view.end_view(cx);
        }
        
        // if let Some(xr_event) = &self.last_xr_update{
        if self.ray_view.begin_view(cx, Layout::abs_origin_zero()).is_ok() {
            let ray_size = Vec3 {x: 0.02, y: 0.02, z: 0.12};
            let ray_pos = Vec3 {x: 0., y: 0., z: 0.0};
            
            let empty_mat = Mat4::identity();
            self._left_ray_area = self.ray_cube.draw_cube(cx, ray_size, ray_pos, &empty_mat).into();
            self._right_ray_area = self.ray_cube.draw_cube(cx, ray_size, ray_pos, &empty_mat).into();
            
            self.ray_view.end_view(cx);
        }
        
    }
}
