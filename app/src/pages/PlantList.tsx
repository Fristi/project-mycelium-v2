import { Leaf } from "lucide-react";
import Retrieve from "../Retrieve";
import { createRetriever } from "../api";
import { Station } from "../backend-client/api";
import PlantCard from "../components/PlantCard";

export const PlantList = () => {
  const renderData = (stations: Array<Station>) => {
    if(!stations || !stations.length) return (<p>No stations here..</p>)

    return (
      <div>
        

        <div className="flex items-center gap-2 text-green-600 dark:text-green-400">
          <Leaf className="h-5 w-5" />
          <p className="text-base/8 font-semibold">Your lushies..!</p>
        </div>

        <h1 className="mt-4 text-base font-semibold tracking-tight text-pretty text-gray-900 sm:text-6xl dark:text-white">
          Your plants
        </h1>
        
        <div className="mx-auto mt-16 grid max-w-2xl grid-cols-1 gap-x-8 gap-y-20 lg:mx-0 lg:max-w-none lg:grid-cols-3">
          {stations.map((s) => (
            <PlantCard key={s.id} station={s} />
          ))}
        </div>
      </div>
    );
  };

  const retriever = createRetriever(x => x.listStations());

  return <Retrieve dataKey="stations" retriever={retriever} renderData={renderData} />
};
