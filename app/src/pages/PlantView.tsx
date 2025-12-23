import { useParams } from "react-router-dom";
import AreaGraph from "../components/AreaGraph";
import { createRetriever } from "../api";
import Retrieve from "../Retrieve";
import PlantLocation from "../components/PlantLocation";

import { StationDetails, StationLog, StationMeasurement } from "../backend-client/api";

type PlantLogProps = { plantId: string };

const PlantLog = (props: PlantLogProps) => {

  const renderPlantLog = (log: Array<StationLog>) => {
    return (
      <div className="flow-root">
        <ul role="list" className="-mb-8">
          <li>Empty</li>
        </ul>
      </div>
    );
  };

  const getStationLog = createRetriever(x => x.getStationLog(props.plantId));

  return <Retrieve dataKey={`plants/${props.plantId}/log`} retriever={getStationLog} renderData={renderPlantLog} />;
};

export const PlantView = () => {
  const { plantId } = useParams();

  const splitMeasurements = (data?: Array<StationMeasurement>) => {
    return {
      batteryVoltage: data?.map((x) => ({ on: x.on, value: x.batteryVoltage })) ?? [],
      humidity: data?.map((x) => ({ on: x.on, value: x.humidity })) ?? [],
      lux: data?.map((x) => ({ on: x.on, value: x.lux })) ?? [],
      soilPf: data?.map((x) => ({ on: x.on, value: x.soilPf })) ?? [],
      tankPf: data?.map((x) => ({ on: x.on, value: x.tankPf })) ?? [],
      temperature: data?.map((x) => ({ on: x.on, value: x.temperature })) ?? [],
    };
  };

  const getStationDetails = createRetriever(x => x.getStation(plantId ?? ""));

  const renderData = (stationDetails: StationDetails) => {
    const station = stationDetails.station;
    const plantId = station.id;
    const host = import.meta.env.VITE_API_BASE_URL || "http://localhost:8080/api";
    const measurements = splitMeasurements(stationDetails.measurements);

    return (
      <>
        <div className="bg-white shadow sm:rounded-lg">
          <div className="px-4 sm:px-6 lg:mx-auto lg:px-8">
            <div className="py-6 md:flex md:items-center md:justify-between lg:border-t lg:border-gray-200">
              <div className="flex-1 min-w-0">
                <div className="flex items-center">
                  <img className="hidden h-16 w-16 rounded-full sm:block" src={`${host}/stations/${plantId}/avatar`} alt="" />
                  <div>
                    <div className="flex items-center">
                      <img className="h-16 w-16 rounded-full sm:hidden" src={`${host}/stations/${plantId}/avatar`} alt="" />
                      <div className="pl-7">
                        <h1 className="text-2xl font-bold leading-7 text-gray-900 sm:leading-9 sm:truncate">{station.name}</h1>
                        <p>
                          <PlantLocation location={station.location} />
                        </p>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
              <div className="mt-6 flex space-x-3 md:mt-0 md:ml-4">
                <a
                  href={`/#/plants/${station.id}/edit`}
                  className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-cyan-500"
                >
                  Settings
                </a>
              </div>
            </div>
          </div>
        </div>

        <main>
          <div className="mx-auto max-w-7xl py-4">
            <div className="mx-auto grid max-w-2xl grid-cols-1 grid-rows-1 items-start gap-x-8  lg:mx-0 lg:max-w-none lg:grid-cols-3">
              <div className="sm:mx-0 lg:col-span-2 lg:row-span-2 lg:row-end-2">
                <AreaGraph header="Soil capacitive" label="pF" data={measurements.soilPf} />
                <AreaGraph header="Relative humidity" label="%" data={measurements.humidity} />
                <AreaGraph header="Temperature" label="Celsius" data={measurements.temperature} />
                <AreaGraph header="Lux" label="lx" data={measurements.lux} />
                <AreaGraph header="Watertank capacitive" label="pF" data={measurements.tankPf} />
                <AreaGraph header="Battery voltage" label="V" data={measurements.batteryVoltage} />
              </div>
              <div className="lg:col-start-3">
                <h2 className="text-sm font-semibold leading-6 text-gray-900 mb-5">Activity</h2>
                <PlantLog plantId={plantId} />
              </div>
            </div>
          </div>
        </main>
      </>
    );
  };

  return <Retrieve dataKey={`plant/${plantId}/details`} retriever={getStationDetails} renderData={renderData} />;
};
