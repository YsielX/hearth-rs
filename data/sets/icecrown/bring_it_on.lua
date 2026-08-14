local card={api_version=1,id="ICC_837",name="Bring It On!",text="Gain 10 Armor. Reduce the Cost of minions in your opponent's hand by (2).",set="ICECROWN",type="spell",class="warrior",rarity="epic",cost=2}
function card.on_play(ctx,self)local p=ctx:controller(self);ctx:gain_armor(p,10);for _,e in ipairs(ctx:hand(ctx:opponent(p)))do if ctx:entity(e).type=="minion"then ctx:modify(e,{stat="cost",operation="add",value=-2})end end end
return card
