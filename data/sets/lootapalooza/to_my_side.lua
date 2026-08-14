local card={api_version=1,id="LOOT_217",name="To My Side!",text="Summon an Animal Companion, or 2 if your deck has no minions.",set="LOOTAPALOOZA",type="spell",class="hunter",rarity="epic",cost=6}
local companions={"NEW1_032","NEW1_033","NEW1_034"}
local function next_one(ctx,self)if(ctx:get_data(self,"companions_left")or 0)>0 and #ctx:board(ctx:controller(self))<7 then ctx:random_value(companions,"companion_chosen")end end
function card.on_play(ctx,self)local n=2;for _,e in ipairs(ctx:deck(ctx:controller(self)))do if ctx:entity(e).type=="minion"then n=1;break end end;ctx:set_data(self,"companions_left",n);next_one(ctx,self)end
function card.companion_chosen(ctx,self,id)ctx:summon(ctx:controller(self),id);ctx:set_data(self,"companions_left",ctx:get_data(self,"companions_left")-1);ctx:continue_with("companion_continue")end
function card.companion_continue(ctx,self)next_one(ctx,self)end
return card
