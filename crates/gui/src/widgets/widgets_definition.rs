pub type WidgetId = u64;

pub trait Widget {
    fn id(&self) -> WidgetId;
}
