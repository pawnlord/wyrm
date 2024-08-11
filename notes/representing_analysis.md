# Representing Analysis
These are some rants and ravings on how to represent the analysis that the decompiler will do  

## timeline
The first idea I had made it really hard to start working: try to represent the structure as a single, mutable structure. But it became increasingly obvious that analysis is best done as an induciton sort of thing: we find the best analysis for instruction 1, and then at any instruction n, we do the best analysis we can for instruction n + 1 (for suitable definitions of '+ 1'). This essentially represents the idea of "forward-flow (all | any) path analysis". For any path analysis, '+ 1' represents "the next instruction that could, potentially, be ran during the program". For all path analysis, it's 'for all instructions that could be step n + 1 of the program."  
I will now dump concerns I come up with this strategy here:
- What about backwards flow analysis?
    - That can be treated as a forward-flow in reverse. No one said the timeline had to be chronological!
- What about references?
    - A problem I found with other methods is that remembering what we're talking about is an issue if everythings mutable. But, if things are only semi-mutable, it becomes much easier: we only need to worry about it at the end of each analysis. During the analysis, we can keep a list of where things are, and then move them when needed.