import { useState, type ChangeEvent } from "react";
import { Card, CardContent } from "@/components/ui/card";
import { Button } from "@/components/ui/button";

export type Detections = [BoundingBox, string, number][];

export interface BoundingBox {
  x1: number;
  x2: number;
  y1: number;
  y2: number;
}

export default function App() {
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [previewUrl, setPreviewUrl] = useState<string | null>(null);
  const [classification, setClassification] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  const handleFileChange = (event: ChangeEvent<HTMLInputElement>) => {
    const files = event.target.files;
    if (!files) {
      return;
    }

    const file = files[0];

    if (file) {
      setSelectedFile(file);
      setPreviewUrl(URL.createObjectURL(file));
      setClassification(null);
    }
  };

  const handleUpload = async () => {
    if (!selectedFile) return;

    setLoading(true);
    const formData = new FormData();
    formData.append("image", selectedFile);

    try {
      const response = await fetch("http://localhost:3000/classify", {
        method: "POST",
        body: formData,
      });

      const result = (await response.json()) as Detections;
      const label = result[0][1];
      const probability = result[0][2];

      setClassification(`${label}: ${probability.toPrecision(5)}`);
    } catch (error) {
      console.error("Error classifying image:", error);
      setClassification("Error");
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="min-h-screen flex flex-col items-center justify-center bg-gray-100 p-4">
      <Card className="w-full max-w-md p-6">
        <CardContent className="flex flex-col gap-4">
          <h1 className="text-2xl font-bold text-center">Image Classifier</h1>

          <input type="file" accept="image/*" onChange={handleFileChange} />

          {previewUrl && (
            <img
              src={previewUrl}
              alt="Preview"
              className="rounded-lg border border-gray-300 mt-2 max-h-64 object-contain"
            />
          )}

          <Button
            onClick={handleUpload}
            disabled={!selectedFile || loading}
            className="mt-4"
          >
            {loading ? "Classifying..." : "Classify Image"}
          </Button>

          {classification && (
            <div className="mt-4 text-center text-lg font-semibold">
              Result: {classification}
            </div>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
