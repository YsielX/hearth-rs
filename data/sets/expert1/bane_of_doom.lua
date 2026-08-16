local card={api_version=1,id="EX1_320",name="Bane of Doom",text="Deal $3 damage to a character. If it dies, summon a random Demon.",set="EXPERT1",type="spell",class="warlock",rarity="epic",spell_school="shadow",cost=5,target_mode="required",targets=function(ctx)return ctx:characters()end}
local function demon(def)for _,t in ipairs(def.tags or{})do if t=="demon"or t=="all"then return def.type=="minion"end end return false end
function card.on_play(ctx,self,target)ctx:set_data(self,"bane_target",target);cardlib.effects.damage(ctx, target,3);ctx:continue_with("bane_check")end
function card.bane_check(ctx,self)local target=ctx:get_data(self,"bane_target");if target and ctx:entity(target).zone~="board"and ctx:entity(target).zone~="hero"then local pool={};for _,id in ipairs(ctx:collectible_cards())do if demon(ctx:card_definition(id))then pool[#pool+1]=id end end;if #pool>0 then ctx:random_value(pool,"bane_summon")end end end
function card.bane_summon(ctx,self,id)ctx:summon(ctx:controller(self),id)end
return card
