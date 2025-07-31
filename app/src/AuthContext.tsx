import { useAuth0, User } from "@auth0/auth0-react";
import React, { useEffect, createContext, useContext } from "react";

type Props = {
  children: React.ReactNode;
};

type ContextType = {
  token?: string;
  user?: User;
};

const initialContext: ContextType = { token: undefined };

const AuthenticationContext = createContext<ContextType>(initialContext);

export const AuthContext: React.FC<Props> = ({ children }) => {
  const { user, isAuthenticated, isLoading, loginWithRedirect, getAccessTokenSilently } = useAuth0();
  const [token, setToken] = React.useState<string | null>(null);

  useEffect(() => {
    (async () => {
      try {
        if (isAuthenticated) {
          const token = await getAccessTokenSilently();
          setToken(token);
        }
      } catch (e) {
        console.error(e);
      }
    })();
  }, [getAccessTokenSilently, isAuthenticated]);

  if (!isAuthenticated || token == null) {
    return (
      <div className="flex flex-col items-center justify-center min-h-screen bg-gray-100 p-4">
        <div className="bg-white shadow-md rounded-lg p-8 max-w-md w-full text-center">
          <h2 className="text-2xl font-bold mb-4 text-gray-800">Authentication Required</h2>
          <p className="text-gray-600 mb-6">
            You need to be logged in to access the app.
          </p>
          <button
            onClick={() => loginWithRedirect()}
            className="bg-emerald-600 hover:bg-emerald-700 text-white font-medium py-2 px-6 rounded-md transition duration-300"
          >
            Sign In
          </button>
        </div>
      </div>
    );
  }

  if (isLoading) {
    return <p>Loading</p>;
  }

  return <AuthenticationContext.Provider value={{ token, user }}>{children}</AuthenticationContext.Provider>;
};

export const useAuth = () => useContext(AuthenticationContext);
