local card={api_version=1,id="UNG_835",name="Chittering Tunneler",text="[x]<b>Battlecry:</b> <b>Discover</b> a spell.\nDeal damage to your hero\nequal to its Cost.",set="UNGORO",type="minion",class="warlock",rarity="epic",cost=2,attack=2,health=2,tags={"beast"},keywords={"battlecry"}}
function card.on_battlecry(ctx,self) local pool={} for _,id in ipairs(ctx:collectible_cards()) do local d=ctx:card_definition(id);if d.type=="spell" and (d.class=="neutral" or d.class=="warlock") then pool[#pool+1]=id end end if #pool>0 then ctx:discover_cards(ctx:controller(self),"Discover a spell",pool,3,"tunneler_chosen") end end
function card.tunneler_chosen(ctx,self,id) local p=ctx:controller(self);ctx:give_card(p,id);ctx:damage(ctx:player(p).hero,ctx:card_definition(id).cost) end
return card
