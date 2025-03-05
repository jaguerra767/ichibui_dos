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


interface LogInProps {
    onUpdate: (newValue: User) => void;
}



const LogIn: React.FC<LogInProps> = ({onUpdate}: LogInProps) => {
    const [otp, setOtp] = useState<string>('');
    //const inputRef = useRef<HTMLInputElement>(null);
    const navigate = useNavigate();

    const test_pw = "1111";

    const logIn = () => {
      console.log(otp);
      if (otp === test_pw) {
        console.log("Password Ok");
        onUpdate(User.Admin);
        navigate('/setup-screen');
      }

    }

    return (
        <Card className="w-[350px]">
            <CardHeader>
                <CardTitle>Log In</CardTitle>
                <CardDescription>Enter your six digit pin</CardDescription>
            </CardHeader>
            <CardContent>
              <div style={{ textAlign: "center" }}>
                <input value={otp} readOnly placeholder="Enter numbers" />
                <div style={{ width: "300px", margin: "auto" }}>
                  <Keyboard
                    onChange={setOtp}
                      layout={{
                        default: ["1 2 3", "4 5 6", "7 8 9", ". 0 {bksp}"]
                      }}
                      display={{
                        "{bksp}": "⌫"
                      }}
                      theme="hg-theme-default hg-layout-numeric"
                  />
                </div>
              </div>
            </CardContent>
            <CardFooter className="flex justify-between">
              <Button className="bg-green-700" onClick={logIn}> Log In</Button>
              <Button className="bg-destructive" onClick={() => navigate('/dispense-screen')}>Cancel</Button>
            </CardFooter>
        </Card>
    );
}

export default LogIn;


