local card={api_version=1,id="EX1_310",name="Doomguard",text="<b>Charge</b>. <b>Battlecry:</b> Discard two random cards.",set="EXPERT1",type="minion",class="warlock",rarity="rare",cost=5,attack=5,health=7,tags={"demon"},keywords={"charge","battlecry"}}
local function next_discard(ctx,self)local hand=ctx:hand(ctx:controller(self));if(ctx:get_data(self,"discards_left")or 0)>0 and #hand>0 then ctx:random_entity(hand,"doomguard_discard")end end
function card.on_battlecry(ctx,self)ctx:set_data(self,"discards_left",2);next_discard(ctx,self)end
function card.doomguard_discard(ctx,self,target)ctx:discard(ctx:controller(self),target);ctx:set_data(self,"discards_left",ctx:get_data(self,"discards_left")-1);ctx:continue_with("doomguard_continue")end
function card.doomguard_continue(ctx,self)next_discard(ctx,self)end
return card
