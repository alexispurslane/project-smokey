# Project Smokey
**Wildfire Forecasting using Machine Learning**

Project Smokey provides wildfire forecasts for any location in the United
States selected by the user in the graphical map application, using weather
forecasts as well as regional climate and vegetation information.

## Technical Details

### Forecasting Model

Instead of using a manually-constructed forecasting model which might not fully
take into account all of the variables or underlying complexity of the
application, as well as requiring extensive time, effort, and expertise to
construct, Project Smokey uses a set of TensorFlow-powered neural network
models, trained on a database of 1.88 million wildfires in the US taking place
over roughly the two decades leading up to 2015 as well as the specific weather
conditions for the days leading up to that wildfire. Essentially, given weather
as an input, the models predict the chance that a wildfire will break out, and
its severity. To get a wildfire *prediction*, then, we just need to feed
*weather* predictions into the wildfire prediction model.

In order to account for hidden differences in each region that we
can't directly pull data for that might effect how weather interacts to produce
wildfires, such as specific differences in climate and vegetation that we don't
have granular-enough data to account for, or regional topography, we bin
(group) wildfires by climate/vegetation region, and train separate models on
each of those groups of wildfires (and associated weather, of course).

### User-End Application

The user-facing end of this application is written entirely in Rust, using the
available GTK3 bindings. It consists of two threads, a GUI thread that runs the
user interface, and a back-end thread that queues up requests for predictions
at the various locations the user clicks on in the map UI, and runs the
appropriate TensorFlow model on each request, returning the predictions to the
user interface to be displayed. We hope that by *running* the TensorFlow models
from Rust, and in a separate thread, some performance gains will be achieved.

### Project Architecture

Overall, then, the project has three "phases":

- [ ] Creation of the user-facing application, which we can plug the models into
- [ ] Data processing, binning, and normalization, in preparation for training
- [ ] Model training