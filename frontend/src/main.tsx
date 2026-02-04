import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { registerTheme } from "echarts";
import {Home} from "src/components/Home/Home";
import SignIn from "src/components/SignIn/SignIn";
import { Settings } from "src/components/Settings";
import { Profile } from "src/components/Profile";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { createBrowserRouter, RouterProvider } from "react-router";
import { ReactQueryClientProvider } from "./utils/react_query_provider";
import { APP_ROUTES } from "src/constants/app_routes";
import theme from "src/assets/echart_themes/theme.json";
import { Canvases } from "src/components/Canvases";
import { Canvas } from "src/components/Canvases/Canvas";

registerTheme("theme", theme);

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
        path: APP_ROUTES.PROFILE,
        element: <Profile />,
      },
      {
        path: APP_ROUTES.CANVASES,
        element: <Canvases />,
      },
      {
        path: APP_ROUTES.CANVAS,
        element: <Canvas />,
      },
    ],
  },
  {
    path: APP_ROUTES.SIGN_IN,
    element: <SignIn />,
  },
]);

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ReactQueryClientProvider>
      <RouterProvider router={router} />
    </ReactQueryClientProvider>
  </StrictMode>,
);
