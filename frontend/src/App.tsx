import { Route, Routes } from "react-router-dom";
import Layout from "./components/Layout";
import CategoriesPage from "./pages/CategoriesPage";
import DashboardPage from "./pages/DashboardPage";
import RecordingDetailPage from "./pages/RecordingDetailPage";
import RecordingsPage from "./pages/RecordingsPage";
import SchedulesPage from "./pages/SchedulesPage";
import SettingsPage from "./pages/SettingsPage";

export default function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route path="/" element={<DashboardPage />} />
        <Route path="/recordings" element={<RecordingsPage />} />
        <Route path="/recordings/:id" element={<RecordingDetailPage />} />
        <Route path="/schedules" element={<SchedulesPage />} />
        <Route path="/categories" element={<CategoriesPage />} />
        <Route path="/settings" element={<SettingsPage />} />
      </Route>
    </Routes>
  );
}
