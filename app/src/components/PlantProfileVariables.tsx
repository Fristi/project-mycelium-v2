import { Leaf } from "lucide-react";
import { PlantProfileVariables } from "../backend-client/api";

export interface PlantProfileVariablesDisplayProps {
  variables: PlantProfileVariables;
}

export const PlantProfileVariablesDisplay: React.FC<PlantProfileVariablesDisplayProps> = ({ variables }) => {
  return (
    <div className="mt-5">
<Leaf className="absolute right-4 bottom-4 text-green-100 w-24 h-24 opacity-70 pointer-events-none" />
<div className="flex flex-wrap gap-3">
      <div className="flex items-center gap-1">
        🌞 <span>Light (µmol): {variables.lightMmol.start}–{variables.lightMmol.end}</span>
      </div>
      <div className="flex items-center gap-1">
        💡 <span>Light (Lux): {variables.lightLux.start}–{variables.lightLux.end}</span>
      </div>
      <div className="flex items-center gap-1">
        🌡️ <span>Temperature: {variables.temperature.start}–{variables.temperature.end}°C</span>
      </div>
      <div className="flex items-center gap-1">
        💧 <span>Humidity: {variables.humidity.start}–{variables.humidity.end}%</span>
      </div>
      <div className="flex items-center gap-1">
        🌱 <span>Soil Moisture: {variables.soilMoisture.start}–{variables.soilMoisture.end}%</span>
      </div>
      <div className="flex items-center gap-1">
        ⚡ <span>Soil EC: {variables.soilEc.start}–{variables.soilEc.end} mS/cm</span>
      </div>
    </div>
    </div>
    
  );
};
