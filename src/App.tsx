import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';


import "./App.css";
import Header from './Header'
import SetupScreen from './SetupScreen';
import Home from './home'
import DispenseScreen from './dispense-screen';
import { DispenseType, Ingredient, UiData, User } from './types';
import { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';


const ArrayBufferToBase64 = (buffer: ArrayBuffer): string => {
  let binary = "";
  const bytes = new Uint8Array(buffer);
  for (let i = 0; i < bytes.byteLength; i++){
      binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary)
};

function App() {
  const [snacks, setSnacks] = useState<Ingredient[]>([]);
  const [selectedIngredient, setSelectedIngredient] = useState<Ingredient | undefined>(undefined);

  const [user, setUser] = useState<User>(User.None)
  const [dispenseType, setDispenseType] = useState<DispenseType>(DispenseType.Classic);

  useEffect(() => {
    const fetchImage = async (filename: string) => {
        try {
            const data: ArrayBuffer = await invoke('get_image', {filename});
            const base64 = ArrayBufferToBase64(data);
            return base64
        } catch (error) {
            console.error("Failed to load SVG:", error);
            return "";
        }
    };

    const fetchIngredients = async () => {
        try {
            const data: UiData[] = await invoke("get_ingredient_data");
            const images: string[] = [];

            for (const d of data) {
              const image = await fetchImage(d.img);
              images.push(`data:image/svg+xml;base64,${image}`);
            }

            const mappedIngredients: Ingredient[] = data.map((d, index) => ({
                id: d.id,
                name: d.label,
                img_filename: d.img,
                base64_img: images[index]
            }));

            setSnacks(mappedIngredients);
        } catch (err) {
            console.error("Failed to fetch ingredients:", err);
        } 
    };
    fetchIngredients();
},[]);


  return (
    <main className="container w-[600px] h-[1024px] bg-slate-950">
      
      <Router>
      <Header currentDispenseType ={dispenseType}  setDispenseType={setDispenseType} user={user}/>
        <Routes>
          <Route path="/" element={<Home setUser={setUser}/>}/>
          <Route path="/setup-screen" element={<SetupScreen snacks={snacks} setIngredient={setSelectedIngredient} setUser={setUser}/>}/>
          <Route path="/dispense-screen" element={<DispenseScreen snack={selectedIngredient} mode={dispenseType}/>}/>
        </Routes>
      </Router>
    </main>
    
  );
}

export default App;
