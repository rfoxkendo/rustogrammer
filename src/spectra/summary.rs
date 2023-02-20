use super::*;
use ndhistogram::value::Sum;
///
/// Summary spectra are useful in visualizing the status of
/// large detector arrays.  The best way to think of a summary
/// spetrum is as an array of one-d spectra where each  of those
/// spectra is a vertical channel strip in a 2-d spectrum.
/// That is the x coordinate of the spectrum represents an element
/// of the detector array and the y coordinate the 1-d spectrum
/// of whatever in that element.
///
/// Summary spectra, like all spectra can be gated since it
/// implements the Spectrum trait.
///
/// Creating a spectrum requires:
///
/// *   An arbitrary set of existing parameters
/// *   The range of the y axis.
/// *   The number of bins on the y axis.
///
/// Note that if these are defaulted the following selections are made:
/// *  y-min is the minimum of the default minima for the parameters.
/// *  y-max is the maximum of the default maxima for the parameters.
/// *  bins  ist the maximum of the default bins for the parameters.
///
/// In the case where any of these is not provided a default for
/// _all_ parameters, the spectrum cannot be created.
///
pub struct Summary {
    applied_gate: SpectrumGate,
    name: String,
    histogram: Hist2D<axis::Uniform, axis::Uniform, Sum>,

    // Parameter information:
    param_names: Vec<String>,
    param_ids: Vec<u32>,
}
// The trait implementation is relatively straightforward:

impl Spectrum for Summary {
    fn check_gate(&mut self, e: &FlatEvent) -> bool {
        self.applied_gate.check(e)
    }
    // Increment the param_ids index gives the x axis value
    // while its value the parameter id.
    // Increment for _all_ valid ids in the event:
    //
    fn increment(&mut self, e: &FlatEvent) {
        for (x, id) in self.param_ids.iter().enumerate() {
            if let Some(y) = e[*id] {
                self.histogram.fill(&(x as f64, y));
            }
        }
    }
    fn gate(&mut self, name: &str, dict: &ConditionDictionary) -> Result<(), String> {
        self.applied_gate.set_gate(name, dict)
    }
    fn ungate(&mut self) {
        self.applied_gate.ungate()
    }
    fn clear(&mut self) {
        for c in self.histogram.iter_mut() {
            *c.value = Sum::new();
        }
    }
}
impl Summary {
    /// This local function takes the minimum of two values which
    /// may not be defined:
    /// * If both v1/v2 are None, the result is None,
    /// * If either v1/v2 are None but not both, the result is the Non_None one.
    /// * If both v1/v2 are Some, the results is Some the minimum of
    /// the two values.
    fn min<T: PartialOrd>(v1: Option<T>, v2: Option<T>) -> Option<T> {
        if v1.is_none() && v2.is_none() {
            None
        } else {
            if v1.is_none() || v2.is_none() {
                if let Some(v1) = v1 {
                    Some(v1)
                } else {
                    Some(v2.unwrap())
                }
            } else {
                // neither are none:

                let v1 = v1.unwrap();
                let v2 = v2.unwrap();
                if v1 < v2 {
                    Some(v1)
                } else {
                    Some(v2)
                }
            }
        }
    }
    /// Same as min but uses max of v1/v2
    fn max<T: PartialOrd>(v1: Option<T>, v2: Option<T>) -> Option<T> {
        if v1.is_none() && v2.is_none() {
            None
        } else {
            if v1.is_none() || v2.is_none() {
                if let Some(v1) = v1 {
                    Some(v1)
                } else {
                    Some(v2.unwrap())
                }
            } else {
                // neither are none:

                let v1 = v1.unwrap();
                let v2 = v2.unwrap();
                if v1 > v2 {
                    Some(v1)
                } else {
                    Some(v2)
                }
            }
        }
    }
    /// Generate the spectrum.
    /// This fails if:
    /// *    Any of the parameters is not defined.
    /// *    Any axis spec is defaulted but none of the parameters
    /// provide a default for it.  See the comments for Summmary
    /// for how these are handled.
    ///
    pub fn new(
        name: &str,
        params: Vec<String>,
        pdict: &ParameterDictionary,
        ylow: Option<f64>,
        yhigh: Option<f64>,
        bins: Option<u32>,
    ) -> Result<Summary, String> {
        let mut low = None;
        let mut high = None;
        let mut nbins = None;

        let mut param_ids = Vec::<u32>::new();
        let mut param_names = Vec::<String>::new();
        for name in params {
            if let Some(p) = pdict.lookup(&name) {
                param_names.push(name);
                param_ids.push(p.get_id());
                let lims = p.get_limits();
                let b = p.get_bins();
                low = Self::min(low, lims.0);
                high = Self::max(high, lims.1);
                nbins = Self::max(nbins, b);
            } else {
                return Err(format!("Parameter {} does not exist", name));
            }
        }
        // Override defaults
        if let Some(yl) = ylow {
            low = Some(yl);
        }
        if let Some(yh) = yhigh {
            high = Some(yh);
        }
        if let Some(b) = bins {
            nbins = Some(b);
        }
        // if any of the Y axis stuff are still None, that's a failure:

        if let None = low {
            return Err(String::from(
                "None of the parameters can default the axis low limit",
            ));
        }
        if let None = high {
            return Err(String::from(
                "None of the parameters can default the axis high limit",
            ));
        }
        if let None = nbins {
            return Err(String::from(
                "None of the parameters can default the bin count",
            ));
        }
        // Unwrap the axis limits.
        let low = low.unwrap();
        let high = high.unwrap();
        let nbins = nbins.unwrap();

        // create/return the spectrum:

        Ok(Summary {
            applied_gate: SpectrumGate::new(),
            name: String::from(name),
            histogram: ndhistogram!(
                axis::Uniform::new(param_names.len(), 0.0, param_names.len() as f64),
                axis::Uniform::new(nbins as usize, low,  high);
                Sum
            ),
            param_names: param_names.clone(),
            param_ids: param_ids.clone(),
        })
    }
}

#[cfg(test)]
mod summary_tests {
    use super::*;
    use ndhistogram::axis::*;
    use std::cell::RefCell; // Needed in gating
    use std::rc::Rc; // Needed in gating.

    #[test]
    fn new_1() {
        // Works -- all parameters have same limits/bins, default:

        let mut pd = ParameterDictionary::new();
        let mut names = Vec::<String>::new();
        for i in 0..10 {
            let name = format!("param{}", i);
            pd.add(&name).unwrap();
            let p = pd.lookup_mut(&name).unwrap();
            p.set_limits(0.0, 1023.0);
            p.set_bins(1024);
            p.set_description("Arbitrary");
            names.push(name);
        }

        let result = Summary::new("summary-test", names.clone(), &pd, None, None, None);
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.applied_gate.gate.is_none());
        assert_eq!(String::from("summary-test"), s.name);
        assert_eq!(names.len(), s.param_names.len());
        assert_eq!(names.len(), s.param_ids.len());
        for (i, n) in names.iter().enumerate() {
            assert_eq!(*n, s.param_names[i]);
            assert_eq!(i + 1, s.param_ids[i] as usize);
        }
        assert_eq!(2, s.histogram.axes().num_dim());
        let x = s.histogram.axes().as_tuple().0.clone();
        let y = s.histogram.axes().as_tuple().1.clone();

        // XAxes are just name size:

        assert_eq!(0.0, *x.low());
        assert_eq!(names.len() as f64, *x.high());
        assert_eq!(names.len() + 2, x.num_bins());

        assert_eq!(0.0, *y.low());
        assert_eq!(1023.0, *y.high());
        assert_eq!(1024 + 2, y.num_bins());
    }
    #[test]
    fn new_2() {
        // can override axis definitions on the y axis:

        // Works -- all parameters have same limits/bins, default:

        let mut pd = ParameterDictionary::new();
        let mut names = Vec::<String>::new();
        for i in 0..10 {
            let name = format!("param{}", i);
            pd.add(&name).unwrap();
            let p = pd.lookup_mut(&name).unwrap();
            p.set_limits(0.0, 1023.0);
            p.set_bins(1024);
            p.set_description("Arbitrary");
            names.push(name);
        }

        let result = Summary::new(
            "summary-test",
            names.clone(),
            &pd,
            Some(-1.0),
            Some(1.0),
            Some(200),
        );
        assert!(result.is_ok());
        let s = result.unwrap();
        assert!(s.applied_gate.gate.is_none());
        assert_eq!(String::from("summary-test"), s.name);
        assert_eq!(names.len(), s.param_names.len());
        assert_eq!(names.len(), s.param_ids.len());
        for (i, n) in names.iter().enumerate() {
            assert_eq!(*n, s.param_names[i]);
            assert_eq!(i + 1, s.param_ids[i] as usize);
        }
        assert_eq!(2, s.histogram.axes().num_dim());
        let x = s.histogram.axes().as_tuple().0.clone();
        let y = s.histogram.axes().as_tuple().1.clone();

        // XAxes are just name size:

        assert_eq!(0.0, *x.low());
        assert_eq!(names.len() as f64, *x.high());
        assert_eq!(names.len() + 2, x.num_bins());

        assert_eq!(-1.0, *y.low());
        assert_eq!(1.0, *y.high());
        assert_eq!(200 + 2, y.num_bins());
    }
    // Now various ways that new fails:

    #[test]
    fn new_3() {
        // Can't let y axis limits/bins default:

        let mut pd = ParameterDictionary::new();
        let mut names = Vec::<String>::new();
        for i in 0..10 {
            let name = format!("param{}", i);
            pd.add(&name).unwrap();
            names.push(name);
        }
        let result = Summary::new(
            "summary-test",
            names.clone(),
            &pd,
            None,
            Some(1.0),
            Some(200),
        );
        assert!(result.is_err());

        let result = Summary::new(
            "summary-test",
            names.clone(),
            &pd,
            Some(-1.0),
            None,
            Some(200),
        );
        assert!(result.is_err());

        let result = Summary::new(
            "summary-test",
            names.clone(),
            &pd,
            Some(-1.0),
            Some(1.0),
            None,
        );
        assert!(result.is_err());
    }
    #[test]
    fn new_4() {
        // Have an unfound name.

        let mut pd = ParameterDictionary::new();
        let mut names = Vec::<String>::new();
        for i in 0..10 {
            let name = format!("param{}", i);
            pd.add(&name).unwrap();
            let p = pd.lookup_mut(&name).unwrap();
            p.set_limits(0.0, 1023.0);
            p.set_bins(1024);
            p.set_description("Arbitrary");
            names.push(name);
        }
        names.push(String::from("No-such-parameter"));
        let result = Summary::new("summary-test", names.clone(), &pd, None, None, None);
        assert!(result.is_err());

    }
}
