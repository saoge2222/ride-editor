use ride_gui::component::component_container::Container;
use ride_gui::component::component_definition::{Component, ComponentId};
use ride_gui::component::component_event::{ComponentEvent, EventResult};
use ride_gui::component::component_layout::{Axis, Constraints, Flex, Rect, Size};
use ride_gui::component::component_style::Color;
use ride_gui::render::render_draw::DrawList;
use ride_gui::render::render_font_system::load_system_font_or_embedded;
use ride_gui::render::render_glyph::GlyphAtlas;
use ride_gui::render::render_pipeline::RenderPipelineContext;
use ride_gui::render::render_text::TextRenderer;
use ride_gui::vulkano_base::vulkano_base_render_loop::{FrameResources, RenderLoop};

const FONT_PIXEL_SIZE: u32 = 20;
const PANEL_SIZE: Size = Size::new(160.0, 90.0);

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let render_loop = RenderLoop::new()?;

    let mut app = DemoApp {
        pipeline: None,
        text: None,
        root: None,
        last_extent: None,
    };

    render_loop.run(move |resources| {
        if !app.ensure(resources) {
            return;
        }

        let mut draw_list = DrawList::new();
        if let Some(root) = &app.root {
            root.draw(&mut draw_list);
        }
        draw_list.rect(620.0, 560.0, 220.0, 120.0, [0.2, 0.6, 0.9, 1.0]);
        draw_list.line([640.0, 720.0], [1100.0, 720.0], 6.0, [0.9, 0.4, 0.2, 1.0]);
        draw_list.circle([900.0, 560.0], 90.0, [0.3, 0.85, 0.4, 1.0]);

        if let Some(pipeline) = &mut app.pipeline {
            let _ = pipeline.record(resources.builder, resources.extent, &draw_list);
        }
        if let Some(text) = &mut app.text {
            let _ = text.draw(
                resources.builder,
                resources.extent,
                40.0,
                60.0,
                "Ride Editor - Vulkan text renderer",
                [0.9, 0.9, 0.95, 1.0],
            );
            let _ = text.draw(
                resources.builder,
                resources.extent,
                40.0,
                92.0,
                "RIDE_FONT_FAMILY selects a system font",
                [0.5, 0.8, 0.9, 1.0],
            );
        }
    })?;

    Ok(())
}

struct DemoApp {
    pipeline: Option<RenderPipelineContext>,
    text: Option<TextRenderer>,
    root: Option<Container>,
    last_extent: Option<[u32; 2]>,
}

impl DemoApp {
    fn ensure(&mut self, resources: &mut FrameResources) -> bool {
        if self.pipeline.is_some() && self.last_extent == Some(resources.extent) {
            return true;
        }

        self.pipeline = RenderPipelineContext::new(
            resources.device.clone(),
            resources.render_pass.clone(),
            resources.extent,
            resources.memory_allocator.clone(),
        )
        .ok();

        let font = load_system_font_or_embedded();
        let atlas = match GlyphAtlas::new(
            resources.device.clone(),
            resources.queue.clone(),
            resources.memory_allocator.clone(),
            &resources.command_allocator,
            &font,
            FONT_PIXEL_SIZE,
        ) {
            Ok(atlas) => atlas,
            Err(_) => return false,
        };
        self.text = TextRenderer::new(
            resources.device.clone(),
            resources.render_pass.clone(),
            resources.extent,
            resources.memory_allocator.clone(),
            atlas,
        )
        .ok();

        let mut root = Container::new(0);
        root.flex = Flex {
            axis: Axis::Vertical,
            gap: 14.0,
            padding: [18.0; 4],
            ..Default::default()
        };
        root.style.background = Color::rgb(0.14, 0.14, 0.17);
        root.add_child(Box::new(Panel::new(1, Color::rgb(0.25, 0.45, 0.75))));
        root.add_child(Box::new(Panel::new(2, Color::rgb(0.55, 0.35, 0.75))));
        root.add_child(Box::new(Panel::new(3, Color::rgb(0.3, 0.7, 0.5))));
        root.layout(Constraints::tight(
            resources.extent[0] as f32,
            resources.extent[1] as f32,
        ));
        root.arrange(Rect::new(0.0, 0.0, resources.extent[0] as f32, resources.extent[1] as f32));
        self.root = Some(root);
        self.last_extent = Some(resources.extent);
        true
    }
}

struct Panel {
    id: ComponentId,
    bounds: Rect,
    size: Size,
    color: Color,
}

impl Panel {
    fn new(id: ComponentId, color: Color) -> Self {
        Self {
            id,
            bounds: Rect::default(),
            size: PANEL_SIZE,
            color,
        }
    }
}

impl Component for Panel {
    fn id(&self) -> ComponentId {
        self.id
    }

    fn bounds(&self) -> Rect {
        self.bounds
    }

    fn layout(&mut self, constraints: Constraints) -> Size {
        constraints.constrain(self.size)
    }

    fn arrange(&mut self, bounds: Rect) {
        self.bounds = bounds;
    }

    fn draw(&self, draw_list: &mut DrawList) {
        draw_list.rect(
            self.bounds.x,
            self.bounds.y,
            self.bounds.width,
            self.bounds.height,
            self.color.to_array(),
        );
    }

    fn handle_event(&mut self, _event: &ComponentEvent) -> EventResult {
        EventResult::IGNORED
    }
}
