# Word Streak Player

This project ideally plays the game "Word Streak - Words with Friends" for automatically. 
This was primarily intended for me to learn the basics of the Rust programming language. 

## Parts

I am a heathen and didn't bother to slit my code up into reasonable parts. So all of the functions are in one massive file in `src/main.rs`.

Ideally, this does a few things:
- It sends a message to my phone to take a screenshot over ADB
- It transfers that screenshot back to my laptop
- It splits that screenshot into a grid of letters
- It recognizes those letters using an image hashing algorithm
- It recognizes special tiles such as double letter or triple word
- It searches through the board recursively to find all words of 10 or fewer letters
- It calculates the score of each word and sorts them based on highest score
- It sends swipe events to the phone to play the game automatically (UNIMPLEMENTED)

All but the last step are implemented naively. Unfortunately, step 3 was written hard-coded style, so it doesn't work with screenshots from phones not the same size as mine. This should be fixed later. I was having trouble finding documentation on sending swipe events over ADB, so the last step is not implemented at this point.
