# Maqam Classification Implementation

[↑ Navigation](../../NAVIGATION.md) | [← Technical Details](../infrastructure.md)

**Purpose**: Complete CNN-based Arabic maqam (musical mode) classifier

**Audience**: AI agents implementing M6.2 (Melody Analysis - Maqam component)

**Includes**:
- MaqamClassifier class with 8 maqam types
- Feature extraction (12-bin chroma + 20 MFCCs)
- CNN training code
- Dataset: Maqam478 reference

---

## 1.4: Complete Maqam Recognition

```python
from sklearn.preprocessing import StandardScaler
from tensorflow import keras
import joblib

class MaqamClassifier:
    """
    Classify Arabic musical mode (maqam) from audio

    Uses:
    - Pitch histogram (12-bin chroma)
    - MFCCs (20 coefficients)
    - CNN classifier
    """

    def __init__(self, model_path="models/maqam_cnn.h5"):
        self.model = keras.models.load_model(model_path)
        self.scaler = joblib.load("models/maqam_scaler.pkl")

        # 8 common maqams
        self.maqam_labels = [
            "Bayati",    # Most common in Quran
            "Rast",      # Second most common
            "Saba",      # Sad, contemplative
            "Hijaz",     # Dramatic, intense
            "Nahawand",  # Minor-like
            "Ajam",      # Major-like
            "Kurd",      # Melancholic
            "Sikah"      # Neutral third
        ]

    def extract_features(self, audio, sr=16000):
        """
        Extract pitch histogram + MFCCs

        Returns: 32-dimensional feature vector
        """
        # 1. Pitch histogram (chroma) - 12 bins
        # Uses constant-Q transform for better pitch resolution
        chroma = librosa.feature.chroma_cqt(
            y=audio,
            sr=sr,
            hop_length=512,
            n_chroma=12
        )
        chroma_mean = np.mean(chroma, axis=1)  # 12-d

        # 2. MFCCs - 20 coefficients
        mfccs = librosa.feature.mfcc(
            y=audio,
            sr=sr,
            n_mfcc=20,
            n_fft=2048,
            hop_length=512
        )
        mfcc_mean = np.mean(mfccs, axis=1)  # 20-d

        # 3. Concatenate
        features = np.concatenate([chroma_mean, mfcc_mean])
        return features  # 32-d vector

    def predict(self, audio, sr=16000):
        """
        Predict maqam with confidence

        Returns: {
            "predicted": str,
            "confidence": float,
            "probability_distribution": dict
        }
        """
        # Extract features
        features = self.extract_features(audio, sr)
        features_scaled = self.scaler.transform([features])

        # Predict probabilities
        probs = self.model.predict(features_scaled, verbose=0)[0]
        predicted_idx = np.argmax(probs)

        return {
            "predicted": self.maqam_labels[predicted_idx],
            "confidence": float(probs[predicted_idx]),
            "probability_distribution": {
                label: float(prob)
                for label, prob in zip(self.maqam_labels, probs)
            }
        }

# ============================================
# TRAINING CODE (One-Time Setup)
# ============================================

def train_maqam_classifier(X_train, y_train, X_val, y_val):
    """
    Train CNN on Maqam478 dataset or similar

    X_train: (N, 32) feature vectors
    y_train: (N,) integer labels 0-7
    """
    from tensorflow.keras import layers, models

    # Convert to one-hot
    num_maqams = 8
    y_train_onehot = keras.utils.to_categorical(y_train, num_maqams)
    y_val_onehot = keras.utils.to_categorical(y_val, num_maqams)

    # Standardize features
    scaler = StandardScaler()
    X_train_scaled = scaler.fit_transform(X_train)
    X_val_scaled = scaler.transform(X_val)

    # CNN architecture
    model = models.Sequential([
        layers.Input(shape=(32,)),
        layers.Dense(128, activation='relu'),
        layers.Dropout(0.3),
        layers.Dense(64, activation='relu'),
        layers.Dropout(0.3),
        layers.Dense(num_maqams, activation='softmax')
    ])

    model.compile(
        optimizer='adam',
        loss='categorical_crossentropy',
        metrics=['accuracy']
    )

    # Train
    history = model.fit(
        X_train_scaled,
        y_train_onehot,
        validation_data=(X_val_scaled, y_val_onehot),
        epochs=50,
        batch_size=32,
        verbose=1
    )

    # Save
    model.save("models/maqam_cnn.h5")
    joblib.dump(scaler, "models/maqam_scaler.pkl")

    return model, scaler, history

# Dataset: Maqam478 (https://github.com/MTG/otmm_makam_recognition_dataset)
# or custom Quranic recitation dataset labeled by experts
```

**Accuracy Target**: >90% on 8-maqam classification (per SOTA research)
**Latency**: 100-200ms per ayah (CNN inference)

---
**Related**: [Prosody Algorithms](prosody.md) | [Architecture M6](../../01-architecture/m6-prosody.md)
