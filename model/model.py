from ultralytics import YOLO

def print_detection_results(results):
    for res in results:
        print(res)

        boxes = res.boxes  # Boxes object for bounding box outputs
        masks = res.masks  # Masks object for segmentation masks outputs
        keypoints = res.keypoints  # Keypoints object for pose outputs
        probs = res.probs  # Probs object for classification outputs
        obb = res.obb  # Oriented boxes object for OBB outputs

        # res.show()  # display to screen
        # res.save(filename="result.jpg")  # save to disk

# Load a model
model = YOLO("yolov8n.pt")

# Perform object detection on an image
results = model("data/golden-retriever-tongue-out.jpg")

print_detection_results(results)

results[0].show()

# Export the model to ONNX format
# path = model.export(format="onnx")  # return path to exported model

