import React from 'react';
import { Ingredient, User } from './types';
import SnackCarousel from './components/snack-carousel';


interface SetupScreenProps{
    snacks: Ingredient[],
    setIngredient: (snack: Ingredient) => void,
    setUser: (user: User) => void
}

const SetupScreen: React.FC<SetupScreenProps> = ({snacks, setIngredient, setUser}) => {

    return (
        <div className='relative'>
                <SnackCarousel snacks={snacks} setSnack={setIngredient} setUser={setUser}/>
        </div>
        
  
    );
};

export default SetupScreen;

// import { BrowserRouter as Router, Routes, Route, useNavigate } from "react-router-dom";
// import { useState } from "react";
// import { Button } from "@/components/ui/button";
// import { Carousel, CarouselContent, CarouselItem } from "@/components/ui/carousel";

// const Header = () => (
//   <header className="p-4 bg-gray-800 text-white text-xl text-center">My App</header>
// );

// const Home = () => {
//   const [selectedImage, setSelectedImage] = useState("/images/default.jpg");
//   const navigate = useNavigate();
  
//   return (
//     <div className="grid grid-cols-2 gap-4 p-4">
//       {/* Left Column */}
//       <div className="flex flex-col items-center">
//         <img src={selectedImage} alt="Selected" className="w-64 h-64 rounded-lg" />
//         <div className="mt-4 flex flex-col gap-2">
//           <Button onClick={() => navigate("/screen1")}>Screen 1</Button>
//           <Button onClick={() => navigate("/screen2")}>Screen 2</Button>
//           <Button onClick={() => navigate("/screen3")}>Screen 3</Button>
//           <Button onClick={() => navigate("/screen4")}>Screen 4</Button>
//         </div>
//       </div>
      
//       {/* Right Column */}
//       <div>
//         <Carousel>
//           <CarouselContent>
//             {["/images/pic1.jpg", "/images/pic2.jpg", "/images/pic3.jpg"].map((img) => (
//               <CarouselItem key={img} onClick={() => setSelectedImage(img)}>
//                 <img src={img} alt="carousel item" className="w-32 h-32 cursor-pointer" />
//               </CarouselItem>
//             ))}
//           </CarouselContent>
//         </Carousel>
//       </div>
//     </div>
//   );
// };

// const Screen = ({ name }) => <div className="p-4 text-center text-2xl">{name}</div>;

// function App() {
//   return (
//     <Router>
//       <div className="flex flex-col h-screen">
//         <Header />
//         <div className="flex-1">
//           <Routes>
//             <Route path="/" element={<Home />} />
//             <Route path="/screen1" element={<Screen name="Screen 1" />} />
//             <Route path="/screen2" element={<Screen name="Screen 2" />} />
//             <Route path="/screen3" element={<Screen name="Screen 3" />} />
//             <Route path="/screen4" element={<Screen name="Screen 4" />} />
//           </Routes>
//         </div>
//       </div>
//     </Router>
//   );
// }

// export default App;
