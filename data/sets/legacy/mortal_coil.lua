local card={api_version=1,id="EX1_302", rarity = "free",name="Mortal Coil",text="Deal $1 damage to a minion. If it dies,\ndraw a card.",set="LEGACY",type="spell",class="warlock",spell_school="shadow",cost=1,target_mode="required",targets=function(ctx)return ctx:minions()end}
function card.on_play(ctx,self,target)cardlib.effects.damage(ctx, target,1);ctx:continue_with_entity("check_death",target)end
function card.check_death(ctx,self,target)if ctx:entity(target).zone=="graveyard"then ctx:draw(ctx:controller(self),1)end end
return card
