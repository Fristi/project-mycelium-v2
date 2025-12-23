import Retrieve from "../Retrieve";
import { createRetriever } from "../api";
import { Station } from "../backend-client/api";
import PlantCard from "../components/PlantCard";

export const PlantList = () => {
  const renderData = (stations: Array<Station>) => {
    return (
      <div className="mx-auto mt-16 grid max-w-2xl grid-cols-1 gap-x-8 gap-y-20 lg:mx-0 lg:max-w-none lg:grid-cols-3">
        {stations.map((s) => (
          <PlantCard key={s.id} station={s} />
        ))}
      </div>
    );
  };

  const retriever = createRetriever(x => x.listStations());

  return <Retrieve dataKey="stations" retriever={retriever} renderData={renderData} />;
};
