## Inputs
- (2) X,Y grid location of lat,lon selected by user
- Previous 3 days weather data:
	- Probability of lightning
	- Mixing ratio (measure of how much water is in the atmopshere, so how wat plants are)
	- Wind speed
	- Wind direction
	- Temperature
	- Probability of precipitation
	- Dew point depression
- Prediction for tomorrow's weather:
	- Probability of lightning
	- Mixing ratio (measure of how much water is in the atmopshere, so how wat plants are)
	- Wind speed
	- Wind direction
	- Temperature
	- Probability of precipitation
	- Dew point depression

## Outputs
- Relative probability of fire **tomorrow** (so probability of fire divided by standard probability of fire — the output would then be something like: "200% greater probability of fire than normal, 156% greater" etc instead of "0.0001% probability of fire")