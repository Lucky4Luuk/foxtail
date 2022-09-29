pub struct RenderPass {
}

impl RenderPass {
    pub fn from_view() -> Self {
        Self {}
    }

    pub fn render(self, renderer: &super::Renderer) -> Result<(), super::RenderError> {
        Ok(())
    }
}
