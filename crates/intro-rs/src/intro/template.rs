pub struct IntroTemplate<Audio, Visual> {
    time: f32,
    audio: Audio,
    visual: Visual
}

impl<Audio: crate::Audio, Visual: crate::Visual> crate::Intro for IntroTemplate<Audio, Visual> {
    fn new() -> Self where Self: Sized {
        let time = 0.0;
        let audio = Audio::new();
        let visual = Visual::new();
        Self { time, audio, visual }
    }

    fn time(&mut self) -> &mut f32 {
        &mut self.time
    }

    fn audio(&self) -> &dyn crate::Audio {
        &self.audio
    }

    fn visual(&self) -> &dyn crate::Visual {
        &self.visual
    }
}
