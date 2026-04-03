import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import { Home } from "src/components/Home/Home";
import SignIn from "src/components/SignIn/SignIn";
import { Settings } from "src/components/Settings";
import { Profile } from "src/components/Profile";
import { ProtectedRoute } from "src/components/ProtectedRoute";
import { createBrowserRouter, RouterProvider } from "react-router";
import { ReactQueryClientProvider } from "./utils/react_query_provider";
import { APP_ROUTES } from "src/constants/app_routes";
import { Canvases } from "src/components/Canvases";
import { Canvas } from "src/components/Canvases/Canvas";
import { ConnectionNew } from "./components/Connection/ConnectionNew";
import { ConnectionEdit } from "./components/Connection/ConnectionEdit/ConnectionEdit";
import { ConversationalAnalytics } from "src/components/ConversationalAnalytics";
import { NewConversationPanel } from "./components/ConversationalAnalytics/NewConversationPanel";
import { ExistingConversationPanel } from "./components/ConversationalAnalytics/ExistingConversationPanel";

const router = createBrowserRouter([
  {
    element: <ProtectedRoute />,
    children: [
      {
        path: APP_ROUTES.INDEX,
        element: <Home />,
      },
      {
        path: APP_ROUTES.CONVERSATIONAL_ANALYTICS,
        element: <ConversationalAnalytics />,
        children: [
          {
            path: APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.INDEX,
            element: <NewConversationPanel />,
          },
          {
            path: APP_ROUTES.CONVERSATIONAL_ANALYTICS_ROUTES.CONVERSATION,
            element: <ExistingConversationPanel />,
          },
        ],
      },
      {
        path: APP_ROUTES.SETTINGS,
        element: <Settings />,
      },
      {
        path: APP_ROUTES.CONNECTION_NEW,
        element: <ConnectionNew />,
      },
      {
        path: APP_ROUTES.CONNECTION_EDIT,
        element: <ConnectionEdit />,
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
