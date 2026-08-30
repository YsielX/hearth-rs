local card={api_version=1,id="EX1_409",name="Upgrade!",text="If you have a weapon, give it +1/+1. Otherwise equip a 1/3 weapon.",set="EXPERT1",type="spell",class="warrior",rarity="rare",cost=1}
function card.on_play(ctx,self)local p=ctx:controller(self);local w=ctx:player(p).weapon;if w then cardlib.effects.buff(ctx, w,1,1)else ctx:equip_weapon(p,"EX1_409t")end end
card.tokens={{id="EX1_409t",name="Heavy Axe",text="",set="EXPERT1",type="weapon",class="warrior",collectible=false,cost=1,attack=1,health=3}}
return card
