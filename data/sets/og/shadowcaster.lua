local card={api_version=1,id="OG_291",name="Shadowcaster",text="<b>Battlecry:</b> Choose a friendly minion. Add a 1/1 copy to your hand that costs (1).",set="OG",type="minion",class="rogue",rarity="epic",cost=5,attack=4,health=4,keywords={"battlecry"},target_mode="required_if_available",targets=function(ctx,self)return ctx:friendly_minions(self)end}
function card.on_battlecry(ctx,self,target)if target then cardlib.effects.give_base_copy_with_stats(ctx, ctx:controller(self),target,1,1,1)end end
return card
