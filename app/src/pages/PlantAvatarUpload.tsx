import React, { useState } from "react";
import { PlantProfile } from "../backend-client/api";
import { useParams } from "react-router";
import { createRetriever } from "../api";
import { useAuth } from "../AuthContext";
import { useNavigate } from "react-router-dom";
import { PlantProfileVariablesDisplay } from "../components/PlantProfileVariables";

enum Step {
  UploadImage,
  SelectProfile,
  Confirm,
}

const PlantAvatarUpload: React.FC<{}> = ({}) => {
  const [step, setStep] = useState<Step>(Step.UploadImage);
  const [selectedFile, setSelectedFile] = useState<File | null>(null);
  const [profiles, setProfiles] = useState<PlantProfile[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const navigate = useNavigate();
  const [selectedProfile, setSelectedProfile] = useState<PlantProfile | null>(null);

  const { plantId } = useParams();
  const auth = useAuth();

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    if (e.target.files && e.target.files[0]) {
      setSelectedFile(e.target.files[0]);
      setProfiles([]);
      setError(null);
    }
  };

  const handleUpload = async () => {
    if (!selectedFile) return;

    setLoading(true);
    setError(null);

    try {
      const response = await createRetriever(
        (x) => x.uploadAvatar(plantId ?? "", selectedFile)
      )(auth.token ?? "");
      setProfiles(response.data);
      setStep(Step.SelectProfile);
    } catch (err: any) {
      setError(err.message || "Something went wrong");
    } finally {
      setLoading(false);
    }
  };

  const handleProfileSelect = (profile: PlantProfile) => {
    setSelectedProfile(profile);
    setStep(Step.Confirm);
  };

  const handleConfirm = () => {
    if (selectedProfile) {
      createRetriever(x => x.setProfile(plantId ?? "", selectedProfile))(auth.token ?? "").then(() => navigate(`/#/plants/${plantId}`));
    }
  };

  return (
    <div className="max-w-md mx-auto p-4 border rounded-md shadow-md bg-white">
      {step === Step.UploadImage && (
        <>
          <h2 className="text-lg font-semibold mb-4">Upload an Image</h2>
          <input
            type="file"
            accept="image/*"
            onChange={handleFileChange}
            className="mb-4 w-full text-sm text-gray-700 border rounded p-2"
          />
          {selectedFile && (
            <div className="mb-4">
              <p>Selected file: {selectedFile.name}</p>
              <button
                onClick={handleUpload}
                disabled={loading}
                className="mt-2 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 disabled:opacity-50"
              >
                {loading ? "Uploading..." : "Upload"}
              </button>
            </div>
          )}
          {error && <p className="text-red-500 mb-4">{error}</p>}
        </>
      )}

      {step === Step.SelectProfile && (
        <>
            <h2 className="text-lg font-semibold mb-4">Select a Profile</h2>
            <ul className="space-y-4">
            {profiles.map((profile) => (
                <li key={profile.name}>
                <button
                    onClick={() => handleProfileSelect(profile)}
                    className="w-full text-left p-4 border rounded hover:bg-gray-50 flex flex-col gap-2"
                >
                    <span className="font-medium">{profile.name}</span>

                    <PlantProfileVariablesDisplay variables={profile.variables} />
                </button>
                </li>
            ))}
            </ul>

          <button
            onClick={() => setStep(Step.UploadImage)}
            className="mt-4 px-4 py-2 border rounded hover:bg-gray-100"
          >
            Back
          </button>
        </>
      )}

      {step === Step.Confirm && selectedProfile && (
        <>
          <h2 className="text-lg font-semibold mb-4">Confirm Selection</h2>
          <p className="mb-4">

            You selected: <strong>{selectedProfile.name}</strong>

            <div className="w-full text-left p-4 border flex flex-col gap-2 mt-4">
                <PlantProfileVariablesDisplay variables={selectedProfile.variables} />
            </div>
            
          </p>
          <div className="flex gap-2">
            <button
              onClick={handleConfirm}
              className="px-4 py-2 bg-green-500 text-white rounded hover:bg-green-600"
            >
              Confirm
            </button>
            <button
              onClick={() => setStep(Step.SelectProfile)}
              className="px-4 py-2 border rounded hover:bg-gray-100"
            >
              Back
            </button>
          </div>
        </>
      )}
    </div>
  );
};

export default PlantAvatarUpload;
