local card={api_version=1,id="UNG_845",name="Igneous Elemental",text="<b>Deathrattle:</b> Add two 1/2 Flame Elementals to your hand.",set="UNGORO",type="minion",class="neutral",rarity="common",cost=3,attack=3,health=3,tags={"elemental"},keywords={"deathrattle"}}
function card.on_deathrattle(ctx,self) local p=ctx:controller(self);ctx:give_card(p,"UNG_809t1");ctx:give_card(p,"UNG_809t1") end
return card
