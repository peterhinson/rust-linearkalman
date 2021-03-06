//! This crate implements a standard linear Kalman filter and smoothing for
//! vectors of arbitrary dimension. Implementation method relies on `rulinalg`
//! library for linear algebra computations. Most inputs and outputs rely
//! therefore on (derived) constructs from
//! [rulinalg](https://athemathmo.github.io/rulinalg/doc/rulinalg/index.html)
//! library, in particular
//! [`Vector<f64>`](https://athemathmo.github.io/rulinalg/doc/rulinalg/vector/struct.Vector.html)
//! and
//! [`Matrix<f64>`](https://athemathmo.github.io/rulinalg/doc/rulinalg/matrix/struct.Matrix.html)
//! structs.
//!
//! Currently, implementation method assumes that Kalman filter is time-invariant
//! and is based on the equations detailed below. Notations in the below equations
//! correspond to annotations in the source code.
//!
//! Measurement and state equations:
//!
//! * `z_{t} = H_{t} x_{t} + v_{t}` where `v_{t} ~ N(0, R_{t})`
//! * `x_{t} = F_{t} x_{t-1} + B_{t} u_{t} + w_{t}` where `w_{t} ~ N(0, Q_{t})`
//!
//! Kalman filter equations:
//!
//! * `P_{t|t-1} = F_{t} P_{t-1|t-1} F'_{t} + Q_{t}`
//! * `x_{t|t-1} = F_{t} x_{t-1|t-1} + B_{t} u_{t}`
//! * `K_{t} = P_{t|t-1} H'_{t} * (H_{t} P_{t|t-1} H'_{t} + R_{t})^{-1}`
//! * `P_{t|t} = (Id - K_{t} H_{t}) * P_{t|t-1}`
//! * `x_{t|t} = x_{t|t-1} + K_{t} * (z_{t} - H_{t} x_{t|t-1})`
//!
//! Kalman smoothing equations:
//!
//! * `J_{t} = P_{t|t} F'_{t} P_{t+1|t}^{-1}`
//! * `x_{t|T} = x_{t|t} + J_{t} * (x_{t+1|T} - x_{t+1|t})`
//! * `P_{t|T} = P_{t|t} + J_{t} * (P_{t+1|T} - P_{t+1|t}) * J'_{t}`
//!
//! Nomenclature:
//!
//! * `(x_{t+1|t}, P_{t+1|t})` will be referred to as predicted state variables.
//! * `(x_{t|t}, P_{t|t})` will be referred to as filtered state variables.
//! * `(x_{t|T}, P_{t|T})` will be referred to as smoothed state variables.
//!
//! For now, it is assumed here that `B_{t}` matrix is null and that `Q_{t},
//! R_{t}, H_{t}` and `F_{t}` matrices are constant over time.
//!
//! Besides real data, algorithm takes as inputs `H`, `R`, `F` and `Q` matrices
//! as well as initial guesses for state mean `x0 = x_{1|0}`and covariance
//! matrix `P0 = P_{1|0}`. Covariance matrix `P0` indicates the uncertainty
//! related to the guess of `x0`.


extern crate rulinalg;

use rulinalg::matrix::{BaseMatrix, Matrix};
use rulinalg::vector::Vector;

/// Container object with values for matrices used in the Kalman filtering
/// procedure as well as initial values for state variables.
///
/// # Fields
/// * `q`: process noise covariance
/// * `r`: measurement noise covariance
/// * `h`: observation matrix
/// * `f`: state transition matrix
/// * `x0`: initial guess for state mean at time 1
/// * `p0`: initial guess for state covariance at time 1
#[derive(Debug)]
pub struct KalmanFilter {
    pub q: Matrix<f64>,   // Process noise covariance
    pub r: Matrix<f64>,   // Measurement noise covariance
    pub h: Matrix<f64>,   // Observation matrix
    pub f: Matrix<f64>,   // State transition matrix
    pub x0: Vector<f64>,  // State variable initial value
    pub p0: Matrix<f64>   // State covariance initial value
}


/// Container with the value of state variable and its covariance. This struct
/// is used throughout all parts of Kalman procedure and may refer to predicted,
/// filtered and smoothed variables depending on the context.
#[derive(Clone, Debug)]
pub struct KalmanState {
    pub x: Vector<f64>,   // State vector
    pub p: Matrix<f64>    // State covariance
}

impl KalmanFilter {
    /// Takes in measurement data and returns filtered data based on specified
    /// `KalmanFilter` struct. `filter` actually returns a 2-uple with the first
    /// coordinate being filtered data whereas the second coordinate is the (a
    /// priori) prediction data that can be used later in the smoothing
    /// procedure.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate rulinalg;
    /// extern crate linearkalman;
    ///
    /// use rulinalg::matrix::Matrix;
    /// use rulinalg::vector::Vector;
    /// use linearkalman::KalmanFilter;
    ///
    /// fn main() {
    ///
    ///   let kalman_filter = KalmanFilter {
    ///     // All matrices are identity matrices
    ///     q: Matrix::from_diag(&vec![1.0, 1.0]),
    ///     r: Matrix::from_diag(&vec![1.0, 1.0]),
    ///     h: Matrix::from_diag(&vec![1.0, 1.0]),
    ///     f: Matrix::from_diag(&vec![1.0, 1.0]),
    ///     x0: Vector::new(vec![1.0, 1.0]),
    ///     p0: Matrix::from_diag(&vec![1.0, 1.0])
    ///   };
    ///
    ///   let data: Vec<Vector<f64>> = vec![Vector::new(vec![1.0, 1.0]),
    ///                                     Vector::new(vec![1.0, 1.0])];
    ///
    ///   let run_filter = kalman_filter.filter(&data);
    ///
    ///   // Due to the setup, filtering is equal to the data
    ///   assert_eq!(data[0], run_filter.0[0].x);
    ///   assert_eq!(data[1], run_filter.0[1].x);
    /// }
    /// ```
    pub fn filter(&self, data: &Vec<Vector<f64>>) -> (Vec<KalmanState>, Vec<KalmanState>) {

        let t: usize = data.len();

        // Containers for predicted and filtered estimates
        let mut predicted: Vec<KalmanState> = Vec::with_capacity(t+1);
        let mut filtered: Vec<KalmanState> = Vec::with_capacity(t);

        predicted.push(KalmanState { x: (self.x0).clone(),
                                     p: (self.p0).clone() });

        for k in 0..t {
            filtered.push(update_step(self, &predicted[k], &data[k]));
            predicted.push(predict_step(self, &filtered[k]));
        }

        (filtered, predicted)
    }

    /// Takes in output from `filter` method and returns smoothed data.
    /// Smoothing procedure uses not only past values as is done by `filter` but
    /// also future values to better predict value of the underlying state
    /// variable. Underlying algorithm is known as Rauch-Tung-Striebel smoother
    /// for a fixed interval. Contrary to the filtering process, incremental
    /// smoothing would require re-running Kalman filter on the entire dataset.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate rulinalg;
    /// extern crate linearkalman;
    ///
    /// use rulinalg::matrix::Matrix;
    /// use rulinalg::vector::Vector;
    /// use linearkalman::KalmanFilter;
    ///
    /// fn main() {
    ///
    ///   let kalman_filter = KalmanFilter {
    ///     // All matrices are identity matrices
    ///     q: Matrix::from_diag(&vec![1.0, 1.0]),
    ///     r: Matrix::from_diag(&vec![1.0, 1.0]),
    ///     h: Matrix::from_diag(&vec![1.0, 1.0]),
    ///     f: Matrix::from_diag(&vec![1.0, 1.0]),
    ///     x0: Vector::new(vec![1.0, 1.0]),
    ///     p0: Matrix::from_diag(&vec![1.0, 1.0])
    ///   };
    ///
    ///   let data: Vec<Vector<f64>> = vec![Vector::new(vec![1.0, 1.0]),
    ///                                     Vector::new(vec![1.0, 1.0])];
    ///
    ///   let run_filter = kalman_filter.filter(&data);
    ///   let run_smooth = kalman_filter.smooth(&run_filter.0, &run_filter.1);
    ///
    ///   // Due to the setup, smoothing is equal to the data
    ///   assert_eq!(data[0], run_smooth[0].x);
    ///   assert_eq!(data[1], run_smooth[0].x);
    /// }
    /// ```
    pub fn smooth(&self,
                  filtered: &Vec<KalmanState>,
                  predicted: &Vec<KalmanState>)
                  -> Vec<KalmanState> {

        let t: usize = filtered.len();
        let mut smoothed: Vec<KalmanState> = Vec::with_capacity(t);

        // Do Kalman smoothing in reverse order
        let mut init = (filtered[t - 1]).clone();
        smoothed.push((filtered[t - 1]).clone());

        for k in 1..t {
            smoothed.push(smoothing_step(self, &init,
                                         &filtered[t-k-1],
                                         &predicted[t-k]));
            init = (&smoothed[k]).clone();
        }

        smoothed.reverse();
        smoothed
    }
}

/// Returns a predicted state variable mean and covariance. If prediction for
/// time `t` is desired, then `KalmanState` object with initial conditions
/// contains state mean and covariance at time `t-1` given information up to
/// time `t-1`.
pub fn predict_step(kalman_filter: &KalmanFilter,
                    init: &KalmanState)
                    -> KalmanState {

    // Predict state variable and covariance
    let xp: Vector<f64> = &kalman_filter.f * &init.x;
    let pp: Matrix<f64> = &kalman_filter.f * &init.p * &kalman_filter.f.transpose() +
        &kalman_filter.q;

    KalmanState { x: xp, p: pp}
}

/// Returns an updated state variable mean and covariance given predicted and
/// observed data. Typically, update step will be called after prediction step,
/// data of which will be consequently used as input in updating.
pub fn update_step(kalman_filter: &KalmanFilter,
                   pred: &KalmanState,
                   measure: &Vector<f64>)
                   -> KalmanState {

    let identity = Matrix::<f64>::identity(kalman_filter.x0.size());

    // Compute Kalman gain
    let k: Matrix<f64> = &pred.p * &kalman_filter.h.transpose() *
        (&kalman_filter.h * &pred.p * &kalman_filter.h.transpose() + &kalman_filter.r)
        .inverse()
        .expect("Kalman gain computation failed due to failure to invert.");

    // Update state variable and covariance
    let x = &pred.x + &k * (measure - &kalman_filter.h * &pred.x);
    let p = (identity - &k * &kalman_filter.h) * &pred.p;

    KalmanState { x: x, p: p }

}

/// Returns a tuple containing updated and predicted estimates (in that order)
/// of the state variable and its covariance. This function might be useful for
/// cases where data is incoming and being updated in real-time so that Kalman
/// filtering is run incrementally. Note that given some initial values for `x`
/// and `P`, `filter_step` makes a prediction and then runs the update step to
/// correct the prediction based on the observed data.
pub fn filter_step(kalman_filter: &KalmanFilter,
                   init: &KalmanState,
                   measure: &Vector<f64>)
                   -> (KalmanState, KalmanState) {

    let pred = predict_step(kalman_filter, init);
    let upd = update_step(kalman_filter, &pred, measure);

    (KalmanState { x: upd.x, p: upd.p }, KalmanState { x: pred.x, p: pred.p })
}


fn smoothing_step(kalman_filter: &KalmanFilter,
                  init: &KalmanState,
                  filtered: &KalmanState,
                  predicted: &KalmanState)
                  -> KalmanState {

    let j: Matrix<f64> = &filtered.p * &kalman_filter.f.transpose() *
        &predicted.p.clone().inverse()
        .expect("Predicted state covariance matrix could not be inverted.");
    let x: Vector<f64> = &filtered.x + &j * (&init.x - &predicted.x);
    let p: Matrix<f64> = &filtered.p + &j * (&init.p - &predicted.p) * &j.transpose();

    KalmanState { x: x, p: p }

}
