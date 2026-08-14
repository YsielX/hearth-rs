local card={api_version=1,id="UNG_833",name="Lakkari Felhound",text="<b>Taunt</b>\n<b>Battlecry:</b> Discard your two lowest-Cost cards.",set="UNGORO",type="minion",class="warlock",rarity="common",cost=4,attack=3,health=8,tags={"demon"},keywords={"taunt","battlecry"}}
local function next_discard(ctx,self) local left=ctx:get_data(self,"felhound_left");local hand=ctx:hand(ctx:controller(self));if left<=0 or #hand==0 then return end local low=100 local pool={} for _,e in ipairs(hand) do local c=ctx:entity(e).cost;if c<low then low=c;pool={e} elseif c==low then pool[#pool+1]=e end end ctx:set_data(self,"felhound_left",left-1);ctx:random_entity(pool,"felhound_discard") end
function card.on_battlecry(ctx,self) ctx:set_data(self,"felhound_left",2);ctx:continue_with("felhound_next") end
function card.felhound_next(ctx,self) next_discard(ctx,self) end
function card.felhound_discard(ctx,self,e) ctx:discard(ctx:controller(self),e);ctx:continue_with("felhound_next") end
return card
