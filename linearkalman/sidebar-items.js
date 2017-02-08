initSidebarItems({"fn":[["filter_step","Returns a tuple containing updated and predicted estimates (in that order) of the state variable and its covariance. This function might be useful for cases where data is incoming and being updated in real-time so that Kalman filtering is run incrementally. Note that given some initial values for `x` and `P`, `filter_step` makes a prediction and then runs the update step to correct the prediction based on the observed data."],["predict_step","Returns a predicted state variable mean and covariance. If prediction for time `t` is desired, then `KalmanState` object with initial conditions contains state mean and covariance at time `t-1` given information up to time `t-1`."],["update_step","Returns an updated state variable mean and covariance given predicted and observed data. Typically, update step will be called after prediction step, data of which will be consequently used as input in updating."]],"struct":[["KalmanFilter","Container object with values for matrices used in the Kalman filtering procedure as well as initial values for state variables."],["KalmanState","Container with the value of state variable and its covariance. This struct is used throughout all parts of Kalman procedure and may refer to predicted, filtered and smoothed variables depending on the context."]]});