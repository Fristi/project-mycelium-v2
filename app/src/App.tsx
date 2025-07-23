import { QueryClient, QueryClientProvider } from "react-query";
import Shell from "./Shell";

const queryClient = new QueryClient();

const App: React.FC = () => {
  // const { handleRedirectCallback } = useAuth0();

  // useEffect(() => {
  //   // Handle the 'appUrlOpen' event and call `handleRedirectCallback`
  //   CapApp.addListener("appUrlOpen", async ({ url }) => {
  //     if (url.includes("state") && (url.includes("code") || url.includes("error"))) {
  //       await handleRedirectCallback(url);
  //     }
  //     // No-op on Android
  //     await Browser.close();
  //   });
  // }, [handleRedirectCallback]);

  return (
    <div>
      <QueryClientProvider client={queryClient}>
        <Shell />
      </QueryClientProvider>
    </div>
  );
};

export default App;
