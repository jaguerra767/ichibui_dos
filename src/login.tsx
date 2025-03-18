import { useNavigate } from "react-router-dom";
import React, { useState } from "react";

import Keyboard  from "react-simple-keyboard";

import { User } from "./types";
import { 
    Card, 
    CardContent, 
    CardDescription, 
    CardFooter, 
    CardHeader, 
    CardTitle 
} from "./components/ui/card";
import "react-simple-keyboard/build/css/index.css";
import { Button } from "./components/ui/button";
import { invoke } from "@tauri-apps/api/core";


interface LogInProps {
    onUpdate: (newValue: User) => void;
}


const LogIn: React.FC<LogInProps> = ({onUpdate}: LogInProps) => {
  const [otp, setOtp] = useState<string>('');
  const [isLoading, setIsLoading] = useState<boolean>(false);
  const navigate = useNavigate();
  
  const logIn = async () => {
      console.log(otp);
      setIsLoading(true);
      
      try {
          // Call the Rust function via Tauri
          const user: User = await invoke('log_in', { pin: otp });
          
          console.log("Login result:", user);
          
          // Check if login was successful
          if (user !== User.None) {
              onUpdate(user);
              navigate('/setup-screen');
          } else {
              // Handle failed login
              console.log("Invalid PIN");
              // Optionally show an error message to the user
              setOtp(''); // Clear the input
          }
      } catch (error) {
          console.error("Login error:", error);
      } finally {
          setIsLoading(false);
      }
  }

    return (
        <Card className="w-[700px] h-[700px]">
            <CardHeader>
              <div className="text-2xl">
                <CardTitle>Log In</CardTitle>
                <CardDescription>Enter Pin Number</CardDescription>
              </div>

            </CardHeader>
            <CardContent>
              <div style={{ textAlign: "center" }}>
                <input className="text-4xl text-center" value={otp} readOnly placeholder="Enter numbers" />
                <div style={{ width: "500px", height: "400px", margin: "auto"}}>
                  <Keyboard
                    onChange={setOtp}
                      layout={{
                        default: ["1 2 3", "4 5 6", "7 8 9", "0 {bksp}"]
                      }}
                      display={{
                        "{bksp}": "del"
                      }}
                      buttonTheme={[
                        {
                          class: "text-3xl font-bold cursor-none", // Tailwind classes for all keys
                          buttons: "1 2 3 4 5 6 7 8 9 0",
                        },
                        {
                          class: "text-3xl cursor-none", // Tailwind classes for all keys
                          buttons: "{bksp}",
                        },
                      ]}
                      theme="hg-theme-default hg-layout-numeric cursor-none"
                    
                  />
                </div>
              </div>
            </CardContent>
            <CardFooter className="flex justify-between">
              <Button className=" m-auto text-4xl bg-green-700 w-[200px] h-[100px]" onClick={logIn} disabled={isLoading || otp.length === 0}> {isLoading ? "Logging in..." : "Log In"}</Button>
              <Button className="m-auto text-4xl bg-destructive w-[200px] h-[100px]" onClick={() => navigate(-1)} disabled={isLoading}>Cancel</Button>
            </CardFooter>
        </Card>
    );
}

export default LogIn;


