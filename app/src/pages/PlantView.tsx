import { useParams } from "react-router-dom";
import AreaGraph from "../components/AreaGraph";
import { createRetriever } from "../api";
import Retrieve from "../Retrieve";
import PlantLocation from "../components/PlantLocation";

import { PlantProfile, StationDetails, StationMeasurement } from "../backend-client/api";
import { Activity, Leaf } from "lucide-react";
import { PlantProfileDisplay } from "../components/PlantProfile";




export const PlantView = () => {
  const { plantId } = useParams();

  const NoProfileCard = () => {
    return (
      <div className="relative bg-white rounded-2xl shadow-md p-6 overflow-hidden mx-auto">
        {/* Background Icon */}
        <Leaf className="absolute right-4 bottom-4 text-green-100 w-24 h-24 opacity-70 pointer-events-none" />

        {/* Content */}
        <div className="relative z-10 flex flex-col items-center text-center">
          <h2 className="text-xl font-semibold text-gray-800 mb-2">
            No Profile detected
          </h2>
          <p className="text-gray-500 text-sm">
            You haven't selected a plant profile yet. Upload an picture of your plant and we'll try to classify it
          </p>
          <a href={`/#/plants/${plantId ?? ""}/avatar`} className="mt-4 px-4 py-2 bg-green-500 text-white rounded-lg hover:bg-green-600 transition">
            Upload picture
          </a>
        </div>
      </div>
    );
  }

  const NoMeasurements = () => {
    return (
      <div className="lg:mt-20 md:mt-4 mb-4 max-w-lg">
      <div className="flex items-center gap-2 text-green-600 dark:text-green-400">
        <Activity className="h-5 w-5" />
        <p className="text-base/8 font-semibold">No data</p>
      </div>

      <h1 className="mt-4 text-1xl font-semibold tracking-tight text-pretty text-gray-900 sm:text-6xl dark:text-white">
        No measurements
      </h1>

      <p className="mt-6 text-lg font-medium text-pretty text-gray-500 sm:text-xl/8 dark:text-gray-400">
        Once measurements are recorded, they’ll appear here with detailed insights
        and trends over time.
      </p>

      <div className="mt-10">
        <button
          type="button"
          className="text-sm/7 font-semibold text-green-600 dark:text-green-400 hover:underline"
        >
          Start your first measurement <span aria-hidden="true">→</span>
        </button>
      </div>
    </div>
    )
  }

  const splitMeasurements = (data?: Array<StationMeasurement>) => {
    if(data && data?.length > 0) {
      return {
        batteryVoltage: data.map((x) => ({ on: x.on, value: x.batteryVoltage })),
        humidity: data.map((x) => ({ on: x.on, value: x.humidity })),
        lux: data.map((x) => ({ on: x.on, value: x.lux })),
        soilPf: data.map((x) => ({ on: x.on, value: x.soilPf })),
        tankPf: data.map((x) => ({ on: x.on, value: x.tankPf })),
        temperature: data.map((x) => ({ on: x.on, value: x.temperature })),
      };
    } else {
      return undefined
    }
  };

  const renderProfile = (data?: PlantProfile) => {
    if(data) {
      return <PlantProfileDisplay profile={data} />
    } else {
      return <NoProfileCard />
    }
  };

  const getStationDetails = createRetriever(x => x.getStation(plantId ?? ""));
  const getPlantProfile = createRetriever(x => x.getStationProfile(plantId ?? ""));

  const renderMeasurement = (measurements: any) => {
    return (
      <>
        <AreaGraph header="Soil capacitive" label="pF" data={measurements.soilPf} />
        <AreaGraph header="Relative humidity" label="%" data={measurements.humidity} />
        <AreaGraph header="Temperature" label="Celsius" data={measurements.temperature} />
        <AreaGraph header="Lux" label="lx" data={measurements.lux} />
        <AreaGraph header="Watertank capacitive" label="pF" data={measurements.tankPf} />
        <AreaGraph header="Battery voltage" label="V" data={measurements.batteryVoltage} />
      </>
    )
  }

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
                <a
                  href={`/#/plants/${station.id}/avatar`}
                  className="inline-flex items-center px-4 py-2 border border-gray-300 shadow-sm text-sm font-medium rounded-md text-gray-700 bg-white hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-cyan-500"
                >
                  Upload image
                </a>
              </div>
            </div>
          </div>
        </div>

        <main>
          <div className="mx-auto max-w-7xl py-4">
            <div className="mx-auto grid max-w-2xl grid-cols-1 grid-rows-1 items-start gap-x-8  lg:mx-0 lg:max-w-none lg:grid-cols-3">
              <div className="sm:mx-0 lg:col-span-2 lg:row-span-2 lg:row-end-2">
                {measurements ? renderMeasurement(measurements) : (<NoMeasurements />)}
              </div>
              <div className="lg:col-start-3">
                <Retrieve dataKey={`plant/${plantId}/profile`} retriever={getPlantProfile} renderData={renderProfile} />
              </div>
            </div>
          </div>
        </main>
      </>
    );
  };

  return <Retrieve dataKey={`plant/${plantId}/details`} retriever={getStationDetails} renderData={renderData} />
};