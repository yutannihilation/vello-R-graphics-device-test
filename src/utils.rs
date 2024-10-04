impl From<&crate::StrokeParameters> for vello::kurbo::Stroke {
    fn from(value: &crate::StrokeParameters) -> Self {
        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
        let join = match value.join {
            1 => vello::kurbo::Join::Round,
            2 => vello::kurbo::Join::Miter,
            3 => vello::kurbo::Join::Bevel,
            v => panic!("invalid join value: {v}"),
        };
        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L183-L187
        let cap = match value.cap {
            1 => vello::kurbo::Cap::Round,
            2 => vello::kurbo::Cap::Round,
            3 => vello::kurbo::Cap::Round,
            v => panic!("invalid cap value: {v}"),
        };

        // cf. https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/include/R_ext/GraphicsEngine.h#L413C1-L419C50
        //
        // TODO: I need to figure out the conversion logic. What is this `& 15`...?
        //
        // https://github.com/r-devel/r-svn/blob/6ad1e0f2702fd0308e4f3caac2e22541d014ab6a/src/modules/X11/devX11.c#L1224
        // https://github.com/r-lib/ragg/blob/6e8bfd1264dfaa36aa6f92592e13a1169986e7b9/src/AggDevice.h#L195C8-L205
        let dash_pattern = match value.linetype {
            -1 => Default::default(), // TODO
            0 => Default::default(),
            49 => vello::kurbo::Dashes::from_const([1.0, 1.0, 1.0, 1.0]), // LTY_DOTTED	1 + (3<<4)
            68 => vello::kurbo::Dashes::from_const([1.0, 1.0, 1.0, 1.0]), // LTY_DASHED	4 + (4<<4)
            _ => Default::default(),
        };
        Self {
            width: value.width,
            join,
            miter_limit: value.miter_limit,
            start_cap: cap,
            end_cap: cap,
            dash_pattern,
            dash_offset: 0.0,
        }
    }
}