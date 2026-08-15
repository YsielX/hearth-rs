local card={api_version=1,id="EX1_082",name="Mad Bomber",text="<b>Battlecry:</b> Deal 3 damage randomly split between all other characters.",set="EXPERT1",type="minion",rarity="common",cost=2,attack=3,health=2,keywords={"battlecry"}}
local function next(ctx,self)local n=ctx:get_data(self,"bombs");if n<=0 then return end;local r={};for _,e in ipairs(ctx:characters())do if e~=self then r[#r+1]=e end end;if #r>0 then ctx:set_data(self,"bombs",n-1);ctx:random_entity(r,"hit")end end
function card.on_battlecry(ctx,self)ctx:set_data(self,"bombs",3);next(ctx,self)end
function card.hit(ctx,self,target)ctx:damage_ignoring_spell_damage(target,1);ctx:continue_with("next_bomb")end
function card.next_bomb(ctx,self)next(ctx,self)end
return card
