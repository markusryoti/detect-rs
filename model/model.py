from ultralytics import YOLO


# Load a model
model = YOLO("yolov8n.pt")

# Perform object detection on an image
results = model("data/kitten.png")
results[0].show()

# Export the model to ONNX format
# path = model.export(format="onnx")  # return path to exported model

