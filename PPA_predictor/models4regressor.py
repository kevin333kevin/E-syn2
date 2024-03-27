# regressor.py
from sklearn.ensemble import GradientBoostingRegressor
import joblib

# Define the regressor
class GradientBoostingRegressorModel:
    def __init__(self, n_estimators=100, learning_rate=0.1, max_depth=3):
        self.model = GradientBoostingRegressor(
            n_estimators=n_estimators,
            learning_rate=learning_rate,
            max_depth=max_depth,
            random_state=0
        )

    def fit(self, X, y):
        self.model.fit(X, y)

    def predict(self, X):
        return self.model.predict(X)

# Save and load functions for the regressor
def save_regressor(regressor, filename):
    joblib.dump(regressor.model, filename)

def load_regressor(filename):
    model = joblib.load(filename)
    return GradientBoostingRegressorModel().model.set_params(**model.get_params())