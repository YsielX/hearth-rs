local card={api_version=1,id="EX1_384",name="Avenging Wrath",text="Deal $8 damage randomly split among all enemies.",set="EXPERT1",type="spell",class="paladin",rarity="epic",spell_school="holy",cost=6}
local function next_hit(ctx,self)if(ctx:get_data(self,"hits_left")or 0)<=0 then return end;local enemies=ctx:enemy_characters(self);if #enemies>0 then ctx:random_entity(enemies,"wrath_hit")end end
function card.on_play(ctx,self)ctx:set_data(self,"hits_left",8);next_hit(ctx,self)end
function card.wrath_hit(ctx,self,target)ctx:damage_ignoring_spell_damage(target,1);ctx:set_data(self,"hits_left",ctx:get_data(self,"hits_left")-1);ctx:continue_with("wrath_continue")end
function card.wrath_continue(ctx,self)next_hit(ctx,self)end
return card
