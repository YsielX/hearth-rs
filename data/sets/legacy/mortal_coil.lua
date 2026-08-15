local card={api_version=1,id="EX1_302",name="Mortal Coil",text="[x]Deal $1 damage to a minion.\n If it dies, draw a card.",set="LEGACY",type="spell",class="warlock",spell_school="shadow",cost=1,target_mode="required",targets=function(ctx)return ctx:minions()end}
function card.on_play(ctx,self,target)ctx:damage(target,1);ctx:continue_with_entity("check_death",target)end
function card.check_death(ctx,self,target)if ctx:entity(target).zone=="graveyard"then ctx:draw(ctx:controller(self),1)end end
return card
