local card={api_version=1,id="EX1_102",name="Demolisher",text="At the start of your turn, deal 2 damage to a random enemy.",set="EXPERT1",type="minion",rarity="rare",cost=3,attack=1,health=4,tags={"mech"},triggers={{event="turn_started",timing="after",active_zones={"board"},condition=function(ctx,self,e)return e.player==ctx:controller(self)end,effect=function(ctx,self)local r=ctx:enemy_characters(self);if #r>0 then ctx:random_entity(r,"hit_selected")end end}}}
function card.hit_selected(ctx,self,target)ctx:damage(target,2)end
return card
