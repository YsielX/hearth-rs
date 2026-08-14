local card={api_version=1,id="LOOT_041",name="Kobold Barbarian",text="At the start of your turn, attack a random enemy.",set="LOOTAPALOOZA",type="minion",class="warrior",rarity="rare",cost=3,attack=4,health=4}
card.triggers={{event="turn_started",timing="after",active_zones={"board"},condition=function(ctx,self,event)return event.player==ctx:controller(self)end,effect=function(ctx,self)local enemies=ctx:enemy_characters(self);if #enemies>0 then ctx:random_entity(enemies,"attack_random_enemy")end end}}
function card.attack_random_enemy(ctx,self,target)ctx:force_attack(self,target)end
return card
