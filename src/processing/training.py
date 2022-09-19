from time import time
import tensorflow as tf
from tensorflow import keras
from keras import layers

import numpy as np
import pandas as pd

column_names = [
# Parameters for the neural network
    'Bin X',
    'Bin Y',

    # Day three days before prediction, two days before pred request
    'Day T-3 Lightning Prob',
    'Day T-3 Mixing Ratio',
    'Day T-3 Wind Speed',
    'Day T-3 Wind Dir',
    'Day T-3 Temp',
    'Day T-3 Precip Prob',

    # Day two days before prediction, one day before pred request
    'Day T-2 Lightning Prob',
    'Day T-2 Mixing Ratio',
    'Day T-2 Wind Speed',
    'Day T-2 Wind Dir',
    'Day T-2 Temp',
    'Day T-2 Precip Prob',
    
    # Day before the prediction and the same day as the request is made
    'Day T-1 Lightning Prob',
    'Day T-1 Mixing Ratio',
    'Day T-1 Wind Speed',
    'Day T-1 Wind Dir',
    'Day T-1 Temp',
    'Day T-1 Precip Prob',

    # Predicted weather properties of day of prediction
    'Day T-0 Lightning Prob',
    'Day T-0 Mixing Ratio',
    'Day T-0 Wind Speed',
    'Day T-0 Wind Dir',
    'Day T-0 Temp',
    'Day T-0 Precip Prob',

# Training output for neural network
    'Was Wildfire'
    ]

dataset = pd.read_csv('training_data.csv',
                      names=column_names,
                      na_values='?',
                      comment='\t',
                      sep=' ',
                      skipinitialspace=True)
dataset = dataset.dropna()
train_dataset = dataset.sample(frac=0.8, random_state=0)
test_dataset = dataset.drop(train_dataset.index)

# Separate the target value we're learning to predict, which is called the
# 'label' from the rest of the data
train_features = train_dataset.copy()
train_labels = train_features.pop('Was Wildfire')

test_features = test_dataset.copy()
test_labels = test_features.pop('Was Wildfire')

norm = layers.Normalization(axis=-1)
norm.adapt(np.array(train_features))
model = keras.Sequential([
    norm,
    layers.Dense(70, activation='relu'),
    layers.Dense(1)
])

model.compile(loss='mean_absolute_error', optimizer=keras.optimizers.Adam(0.001))

%%time
history = model.fit(train_features, train_labels, validation_split=0.2, verbose=1, epochs=100)