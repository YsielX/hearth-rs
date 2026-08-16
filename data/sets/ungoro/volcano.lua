local card={api_version=1,id="UNG_025",name="Volcano",text="Deal $15 damage randomly split among all minions.\n<b>Overload:</b> (1)",set="UNGORO",type="spell",class="shaman",rarity="rare",spell_school="fire",cost=5,keywords={"overload"},keyword_params={overload=1}}
card.rules={can_play=function(ctx) return #ctx:minions()>0 end}
local function next_hit(ctx,self)
    local left=ctx:get_data(self,"volcano_left")
    if left<=0 then return end
    local pool=ctx:minions()
    if #pool==0 then return end
    ctx:set_data(self,"volcano_left",left-1)
    ctx:random_entity(pool,"volcano_hit")
end
function card.on_play(ctx,self) ctx:set_data(self,"volcano_left",15); ctx:continue_with("volcano_next") end
function card.volcano_next(ctx,self) next_hit(ctx,self) end
function card.volcano_hit(ctx,self,target) cardlib.effects.damage_ignoring_spell_damage(ctx, target,1); ctx:continue_with("volcano_next") end
return card
