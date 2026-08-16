local card={api_version=1,id="EX1_133",name="Perdition's Blade",text="<b>Battlecry:</b> Deal 1 damage. <b>Combo:</b> Deal 2 instead.",set="EXPERT1",type="weapon",class="rogue",rarity="rare",cost=3,attack=2,health=2,keywords={"battlecry","combo"},target_mode="required_if_available",targets=function(ctx)return ctx:characters()end}
function card.on_battlecry(ctx,self,target)if target and not ctx:combo_active(self)then cardlib.effects.damage(ctx, target,1)end end
function card.on_combo(ctx,self,target)if target then cardlib.effects.damage(ctx, target,2)end end
return card
