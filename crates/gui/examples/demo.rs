use ride_gui::render::render_draw::DrawList;
use ride_gui::render::render_pipeline::RenderPipelineContext;
use ride_gui::vulkano_base::vulkano_base_render_loop::RenderLoop;
use ride_gui::vulkano_base::vulkano_base_window::WindowContext;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let window_context = WindowContext::new()?;
    let render_loop = RenderLoop::new(window_context)?;

    let device = render_loop.device();
    let render_pass = render_loop.render_pass();
    let memory_allocator = render_loop.memory_allocator();
    let mut pipeline =
        RenderPipelineContext::new(device, render_pass, [1280, 800], memory_allocator)?;

    let mut last_extent = None;
    render_loop.run(move |builder, extent, _image_index| {
        if last_extent != Some(extent) {
            let _ = pipeline.recreate(extent);
            last_extent = Some(extent);
        }

        let mut draw_list = DrawList::new();
        draw_list.rect(100.0, 80.0, 220.0, 140.0, [0.2, 0.6, 0.9, 1.0]);
        draw_list.rect(360.0, 80.0, 220.0, 140.0, [0.4, 0.8, 0.3, 1.0]);
        draw_list.line([120.0, 320.0], [760.0, 320.0], 8.0, [0.9, 0.4, 0.2, 1.0]);
        draw_list.circle([520.0, 520.0], 110.0, [0.8, 0.7, 0.2, 1.0]);
        let _ = pipeline.record(builder, extent, &draw_list);
    })
}
