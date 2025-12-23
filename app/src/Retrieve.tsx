import { useQuery } from "react-query";
import { useAuth } from "./AuthContext";
import { ReactElement } from "react";
import { AxiosResponse } from "axios";

type Props<T> = {
  dataKey: string;
  retriever: (token: string) => Promise<AxiosResponse<T, any>>;
  renderData: (data: T) => ReactElement;
};

export default function <T>(props: Props<T>): ReactElement {
  const auth = useAuth();
  const token = auth.token ?? "";
  // const { status, data, error, isFetching } = useQuery([props.dataKey], () => props.retriever(token));
  const { data } = useQuery([props.dataKey], () => props.retriever(token));

  if (data) {
    return props.renderData(data.data);
  }

  return <></>;
}
