//!  PGamma Spectra are useful  in particle gamma
//!  coincidence experiments where both the gamma and
//!  massive particle detectors are arrays.  
//!  The spectra are defined on two indpendent arrays of
//!  parameters, one for the X and one for the Y axis.
//!  When an event can increment the spectrum, all pairs of X/Y
//!  parameters generate increments.  For example:
//!  consider a fully populated event and a Pgamma
//!   histogram with parameters 1,3 on the x axis and 5,7,8 on the y axis, the following
//!   parameter pairs will be used to increment the spectrum:
//!   (1,5), (1,7), (1,8), (3,5), (3,7), (3,8).
//!
//!  As with any spectrum a condition can be applied to gate the increment
//!  of the spectrum.  That is the condition, applied as the gate must
//!  be true for the event to be eligible to increment the spectrum.
//!
//!  Default axis specification are derived indpendently from the
//!  default axis specification fo the X and Y parameter sets.
//!  The algorithm to choose from among the specification is the same
//!  as for all : minimum *_low, maximum of *_high and *_bins.
//!
use super::*;
use ndhistogram::value::Sum;

// This struct defines a parameter for the spectrum:

struct SpectrumParameter {
    name: String,
    id: u32,
}

///
/// PGamma is the struct that represents the Particle Gamma Spectrum.
/// In addition to the name and histogram it encapsulates  an array
/// of X and an independent array of Y parameters stored as
/// SpectrumParameter objects:
///
pub struct PGamma {
    applied_gate: SpectrumGate,
    name: String,
    histogram: H2DContainer,

    x_params: Vec<SpectrumParameter>,
    y_params: Vec<SpectrumParameter>,
}
// to make this a spectrum we need to implement this trait:

impl Spectrum for PGamma {
    fn check_gate(&mut self, e: &FlatEvent) -> bool {
        self.applied_gate.check(e)
    }
    // Increment the param_ids index gives the x axis value
    // while its value the parameter id.
    // Increment for _all_ valid ids in the event:
    //
    fn increment(&mut self, e: &FlatEvent) {
        let mut histogram = self.histogram.borrow_mut();
        for xp in self.x_params.iter() {
            for yp in self.y_params.iter() {
                let xid = xp.id;
                let yid = yp.id;

                let x = e[xid];
                let y = e[yid];
                if x.is_some() && y.is_some() {
                    histogram.fill(&(x.unwrap(), y.unwrap()));
                }
            }
        }
    }
    fn get_name(&self) -> String {
        self.name.clone()
    }
    fn gate(&mut self, name: &str, dict: &ConditionDictionary) -> Result<(), String> {
        self.applied_gate.set_gate(name, dict)
    }
    fn ungate(&mut self) {
        self.applied_gate.ungate()
    }
    fn get_histogram_1d(&self) -> Option<H1DContainer> {
        None
    }
    fn get_histogram_2d(&self) -> Option<H2DContainer> {
        Some(Rc::clone(&self.histogram))
    }

    fn clear(&mut self) {
        for c in self.histogram.borrow_mut().iter_mut() {
            *c.value = Sum::new();
        }
    }
}
impl PGamma {
    fn make_axis_def(
        params: &Vec<String>,
        pdict: &ParameterDictionary,
    ) -> Result<
        (
            Option<f64>,
            Option<f64>,
            Option<u32>,
            Vec<SpectrumParameter>,
        ),
        String,
    > {
        // Validate all the x parameters and get the x axis default
        // specifications:

        let mut x_min = None;
        let mut x_max = None;
        let mut x_bins = None;
        let mut xp = Vec::<SpectrumParameter>::new();

        for pname in params.iter() {
            if let Some(p) = pdict.lookup(&pname) {
                let lims = p.get_limits();
                x_min = optmin(x_min, lims.0);
                x_max = optmax(x_max, lims.1);
                x_bins = optmax(x_bins, p.get_bins());
                xp.push(SpectrumParameter {
                    name: pname.clone(),
                    id: p.get_id(),
                });
            } else {
                return Err(format!("Parameter {} is not defined", pname));
            }
        }

        Ok((x_min, x_max, x_bins, xp))
    }
    /// Create a new gamma spectrum.   
    /// *   name - the name of the new spectrum.
    /// *   xparams - Vector of x parameter names.
    /// *   yparams - Vector of y parameter names.
    /// *   pdict   - References the parameter dictionary.
    /// *   xmin,xmax,xbins - possible overrides for the x axis specification.
    /// *   ymin,ymax,ybins - possible overrides for the y axis specification.
    ///
    pub fn new(
        name: &str,
        xparams: &Vec<String>,
        yparams: &Vec<String>,
        pdict: &ParameterDictionary,
        xmin: Option<f64>,
        xmax: Option<f64>,
        xbins: Option<u32>,
        ymin: Option<f64>,
        ymax: Option<f64>,
        ybins: Option<u32>,
    ) -> Result<PGamma, String> {
        let xdef = Self::make_axis_def(xparams, pdict);
        if let Err(s) = xdef {
            return Err(s);
        }
        let (mut x_min, mut x_max, mut x_bins, xp) = xdef.unwrap();
        // Override x default axis specs:

        if let Some(_) = xmin {
            x_min = xmin;
        }
        if let Some(_) = xmax {
            x_max = xmax;
        }
        if let Some(_) = xbins {
            x_bins = xbins;
        }

        // All X axis parameters must be defined:

        if x_min.is_none() {
            return Err(String::from("X axis minimum cannot be defaulted"));
        }
        if x_max.is_none() {
            return Err(String::from("X axis maximum cannot be defaulted"));
        }
        if x_bins.is_none() {
            return Err(String::from("X axis bins cannot be defaulted"));
        }
        // Same but for y axis:

        let ydef = Self::make_axis_def(yparams, pdict);
        if let Err(s) = ydef {
            return Err(s);
        }
        let (mut y_min, mut y_max, mut y_bins, yp) = ydef.unwrap();
        if let Some(_) = ymin {
            y_min = ymin;
        }
        if let Some(_) = ymax {
            y_max = ymax;
        }
        if let Some(_) = ybins {
            y_bins = ybins;
        }

        if y_min.is_none() {
            return Err(String::from("Y axis minimum cannot be defaulted"));
        }
        if y_max.is_none() {
            return Err(String::from("Y axis maximum cannot be defaulted"));
        }
        if y_bins.is_none() {
            return Err(String::from("Y axis bins cannot be defaulted"));
        }
        // All good so we can create the return value:

        Ok(PGamma {
            applied_gate: SpectrumGate::new(),
            name: String::from(name),
            histogram: Rc::new(RefCell::new(ndhistogram!(
                axis::Uniform::new(x_bins.unwrap() as usize, x_min.unwrap(), x_max.unwrap()),
                axis::Uniform::new(y_bins.unwrap() as usize, y_min.unwrap(), y_max.unwrap());
                Sum
            ))),
            x_params: xp,
            y_params: yp,
        })
    }
}
#[cfg(test)]
mod pgamma_tests {
    use super::*;
    use ndhistogram::axis::*;
    use std::cell::RefCell; // Needed in gating
    use std::rc::Rc; // Needed in gating.

    fn make_params(n: usize, lh: Option<(f64, f64)>, bins: Option<u32>) -> ParameterDictionary {
        let mut dict = ParameterDictionary::new();
        for i in 0..n {
            let name = format!("param.{}", i);
            dict.add(&name)
                .expect(&format!("Failed to add parameter {}", name));
            let p = dict.lookup_mut(&name).unwrap();
            if let Some((low, high)) = lh {
                p.set_limits(low, high);
            }
            if let Some(b) = bins {
                p.set_bins(b);
            }
        }
        dict
    }
    #[test]
    fn new_1() {
        // Creates ok:

        let dict = make_params(10, Some((0.0, 1024.0)), Some(1024));
        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
        ];

        let result = PGamma::new("test", &xp, &yp, &dict, None, None, None, None, None, None);
        assert!(result.is_ok());
        let spec = result.unwrap();

        assert!(spec.applied_gate.gate.is_none());
        assert_eq!(String::from("test"), spec.name);

        for (i, xp) in spec.x_params.iter().enumerate() {
            let name = format!("param.{}", i);
            assert_eq!(name, xp.name);
            assert_eq!(dict.lookup(&name).unwrap().get_id(), xp.id);
        }
        for (i, yp) in spec.y_params.iter().enumerate() {
            let ii = i + 5;
            let name = format!("param.{}", ii);
            assert_eq!(name, yp.name);
            assert_eq!(dict.lookup(&name).unwrap().get_id(), yp.id);
        }
        // Check out histogram axis defs:

        assert_eq!(2, spec.histogram.borrow().axes().num_dim());
        let x = spec.histogram.borrow().axes().as_tuple().0.clone();
        let y = spec.histogram.borrow().axes().as_tuple().1.clone();

        assert_eq!(0.0, *x.low());
        assert_eq!(1024.0, *x.high());
        assert_eq!(1024 + 2, x.num_bins());

        assert_eq!(0.0, *y.low());
        assert_eq!(1024.0, *y.high());
        assert_eq!(1024 + 2, y.num_bins());
    }
    #[test]
    fn new_2() {
        // Illegal parameter name fails:

        let dict = make_params(10, Some((0.0, 1024.0)), Some(1024));
        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
            String::from("Param.10"), // Undefined y parameter.
        ];

        let result = PGamma::new("test", &xp, &yp, &dict, None, None, None, None, None, None);
        assert!(result.is_err());

        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
            String::from("Param.10"), // Undefined x parameter.
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
        ];
        let result = PGamma::new("test", &xp, &yp, &dict, None, None, None, None, None, None);
        assert!(result.is_err());
    }
    #[test]
    fn new_3() {
        // Can override axis definitions:

        let dict = make_params(10, Some((0.0, 1024.0)), Some(1024));
        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
        ];

        let result = PGamma::new(
            "test",
            &xp,
            &yp,
            &dict,
            Some(-1.0),
            Some(1.0),
            Some(512),
            Some(511.0),
            Some(1000.0),
            Some(256),
        );
        assert!(result.is_ok());
        let spec = result.unwrap();

        // Check out histogram axis defs:

        assert_eq!(2, spec.histogram.borrow().axes().num_dim());
        let x = spec.histogram.borrow().axes().as_tuple().0.clone();
        let y = spec.histogram.borrow().axes().as_tuple().1.clone();

        assert_eq!(-1.0, *x.low());
        assert_eq!(1.0, *x.high());
        assert_eq!(512 + 2, x.num_bins());

        assert_eq!(511.0, *y.low());
        assert_eq!(1000.0, *y.high());
        assert_eq!(256 + 2, y.num_bins());
    }
    #[test]
    fn new_4() {
        // Check for Xlow, Ylow required:

        let dict = make_params(10, None, None); // no default axis specs.
        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
        ];
        let result = PGamma::new(
            "test",
            &xp,
            &yp,
            &dict,
            None,
            Some(1.0),
            Some(512),
            Some(511.0),
            Some(1000.0),
            Some(256),
        );
        assert!(result.is_err());

        let result = PGamma::new(
            "test",
            &xp,
            &yp,
            &dict,
            Some(-1.0),
            Some(1.0),
            Some(512),
            None,
            Some(1000.0),
            Some(256),
        );
        assert!(result.is_err());
    }
    #[test]
    fn new_5() {
        // xhigh/yhigh required:

        let dict = make_params(10, None, None); // no default axis specs.
        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
        ];
        let result = PGamma::new(
            "test",
            &xp,
            &yp,
            &dict,
            Some(-1.0),
            None,
            Some(512),
            Some(511.0),
            Some(1000.0),
            Some(256),
        );
        assert!(result.is_err());

        let result = PGamma::new(
            "test",
            &xp,
            &yp,
            &dict,
            Some(-1.0),
            Some(1.0),
            Some(512),
            Some(511.0),
            None,
            Some(256),
        );
        assert!(result.is_err());
    }
    #[test]
    fn new_6() {
        // Need bins

        let dict = make_params(10, None, None); // no default axis specs.
        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
        ];
        let result = PGamma::new(
            "test",
            &xp,
            &yp,
            &dict,
            Some(-1.0),
            Some(1.0),
            None,
            Some(511.0),
            Some(1000.0),
            Some(256),
        );
        assert!(result.is_err());

        let result = PGamma::new(
            "test",
            &xp,
            &yp,
            &dict,
            Some(-1.0),
            Some(1.0),
            Some(512),
            Some(511.0),
            Some(1000.0),
            None,
        );
        assert!(result.is_err());

        // just to be sure:

        let result = PGamma::new(
            "test",
            &xp,
            &yp,
            &dict,
            Some(-1.0),
            Some(1.0),
            Some(512),
            Some(511.0),
            Some(1000.0),
            Some(256),
        );
        assert!(result.is_ok());
    }
    // Next tests are about incrementing the spectrum.

    #[test]
    fn incr_1() {
        // ungated:

        let dict = make_params(10, Some((0.0, 1024.0)), Some(1024));
        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
        ];

        let mut spec = PGamma::new("test", &xp, &yp, &dict, None, None, None, None, None, None)
            .expect("Failed to make spectruM");

        // Make an event with all parameters present:

        let mut e = Event::new();
        let mut fe = FlatEvent::new();

        let mut all_names = xp.clone();
        for n in yp.iter() {
            all_names.push(n.clone());
        }
        for (i, n) in all_names.iter().enumerate() {
            let value = i as f64 * 10.0;
            let p = dict.lookup(n).unwrap();
            e.push(EventParameter::new(p.get_id(), value));
        }
        fe.load_event(&e);

        spec.handle_event(&fe);

        // All value pairs should have data:
        for (i, _) in xp.iter().enumerate() {
            for (j, _) in yp.iter().enumerate() {
                let x = i as f64 * 10.0;
                let y = (j + 5) as f64 * 10.0;

                let v = spec
                    .histogram
                    .borrow()
                    .value(&(x, y))
                    .expect("Value should exist")
                    .clone();

                assert_eq!(1.0, v.get());
            }
        }
    }
    #[test]
    fn incr_2() {
        // gated on a True gate:

        let dict = make_params(10, Some((0.0, 1024.0)), Some(1024));
        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
        ];

        let mut spec = PGamma::new("test", &xp, &yp, &dict, None, None, None, None, None, None)
            .expect("Failed to make spectrum");

        // Make a true condition and gate the spetrum on it:

        let mut gdict = ConditionDictionary::new();
        assert!(gdict
            .insert(String::from("true"), Rc::new(RefCell::new(True {})))
            .is_none());
        spec.gate("true", &gdict)
            .expect("Could not apply true gate");

        // Make an event with all parameters present:

        let mut e = Event::new();
        let mut fe = FlatEvent::new();

        let mut all_names = xp.clone();
        for n in yp.iter() {
            all_names.push(n.clone());
        }
        for (i, n) in all_names.iter().enumerate() {
            let value = i as f64 * 10.0;
            let p = dict.lookup(n).unwrap();
            e.push(EventParameter::new(p.get_id(), value));
        }
        fe.load_event(&e);

        spec.handle_event(&fe);

        // All value pairs should have data:
        for (i, _) in xp.iter().enumerate() {
            for (j, _) in yp.iter().enumerate() {
                let x = i as f64 * 10.0;
                let y = (j + 5) as f64 * 10.0;

                let v = spec
                    .histogram
                    .borrow()
                    .value(&(x, y))
                    .expect("Value should exist")
                    .clone();

                assert_eq!(1.0, v.get());
            }
        }
    }
    #[test]
    fn incr_3() {
        // Apply a false condition to the spectrum:

        let dict = make_params(10, Some((0.0, 1024.0)), Some(1024));
        let xp = vec![
            String::from("param.0"),
            String::from("param.1"),
            String::from("param.2"),
            String::from("param.3"),
            String::from("param.4"),
        ];
        let yp = vec![
            String::from("param.5"),
            String::from("param.6"),
            String::from("param.7"),
            String::from("param.8"),
            String::from("param.9"),
        ];

        let mut spec = PGamma::new("test", &xp, &yp, &dict, None, None, None, None, None, None)
            .expect("Failed to make spectrum");

        // Make a true condition and gate the spetrum on it:

        let mut gdict = ConditionDictionary::new();
        assert!(gdict
            .insert(String::from("false"), Rc::new(RefCell::new(False {})))
            .is_none());
        spec.gate("false", &gdict)
            .expect("Could not apply false gate");

        // Make an event with all parameters present:

        let mut e = Event::new();
        let mut fe = FlatEvent::new();

        let mut all_names = xp.clone();
        for n in yp.iter() {
            all_names.push(n.clone());
        }
        for (i, n) in all_names.iter().enumerate() {
            let value = i as f64 * 10.0;
            let p = dict.lookup(n).unwrap();
            e.push(EventParameter::new(p.get_id(), value));
        }
        fe.load_event(&e);

        spec.handle_event(&fe);

        // All channels should be zero:

        for c in spec.histogram.borrow().iter() {
            assert_eq!(0.0, c.value.get());
        }
    }
}
