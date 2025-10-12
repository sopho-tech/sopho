import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import Home from "src/components/Home/Home";
import SignUp from "src/components/SignUp/SignUp";
import SignIn from "src/components/SignIn/SignIn";
import { Settings } from "src/components/Settings";
import { Notebooks } from "src/components/Notebooks";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { createBrowserRouter, RouterProvider } from "react-router";
import { ReactQueryClientProvider } from "./utils/react_query_provider";
import { GoogleOAuthProvider } from "@react-oauth/google";
import { APP_ROUTES } from "src/constants/app_routes";
import { Notebook } from "src/components/Notebooks/Notebook";

const router = createBrowserRouter([
  {
    element: <ProtectedRoute />,
    children: [
      {
        path: APP_ROUTES.INDEX,
        element: <Home />,
      },
      {
        path: APP_ROUTES.SETTINGS,
        element: <Settings />,
      },
      {
        path: APP_ROUTES.NOTEBOOKS,
        element: <Notebooks />,
      },
      {
        path: APP_ROUTES.NOTEBOOK,
        element: <Notebook />,
      },
    ],
  },
  {
    path: APP_ROUTES.SIGN_UP,
    element: <SignUp />,
  },
  {
    path: APP_ROUTES.SIGN_IN,
    element: <SignIn />,
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <GoogleOAuthProvider clientId={import.meta.env.VITE_GOOGLE_CLIENT_ID}>
      <ReactQueryClientProvider>
        <RouterProvider router={router} />
      </ReactQueryClientProvider>
    </GoogleOAuthProvider>
  </StrictMode>
);
