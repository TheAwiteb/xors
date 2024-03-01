import { BrowserRouter, Route, Routes } from "react-router-dom";
import { Layout } from "./Components";
import { Ai, Choose, Home, OneVsOne } from "./Pages";

function App() {
  return (
    <>
      <BrowserRouter>
        <Routes>
          <Route path="/" element={<Layout />}>
            <Route index element={<Home />} />
            <Route path="choose" element={<Choose />} />
            <Route path="1v1" element={<OneVsOne />} />
            <Route path="ai" element={<Ai />} />
          </Route>
        </Routes>
      </BrowserRouter>
    </>
  );
}

export default App;
