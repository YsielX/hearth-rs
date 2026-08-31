local card = {
    api_version = 1, id = "ICC_051", name = "Druid of the Swarm",
    text = "<b>Choose One -</b> Transform into a 1/2 with <b>Poisonous</b>; or a 1/5 with <b>Taunt</b>.",
    set = "ICECROWN", type = "minion", class = "druid", rarity = "rare",
    cost = 2, attack = 1, health = 2, keywords = { "choose_one" },
}

function card.on_choose_one(ctx, self)
    ctx:choose_options(ctx:controller(self), "Choose One", {
        { card_id = "ICC_051a", label = "Spider Form" },
        { card_id = "ICC_051b", label = "Scarab Form" },
    }, "swarm_form_chosen")
end

function card.swarm_form_chosen(ctx, self, choice)
    cardlib.effects.transform(ctx, self, choice == "ICC_051a" and "ICC_051t" or "ICC_051t2")
end
function card.on_choose_multiple(ctx, self) cardlib.effects.transform(ctx, self, "ICC_051t3") end

card.tokens = {
    { id = "ICC_051a", rarity = "rare", name = "Spider Form", text = "<b>Poisonous</b>", set = "ICECROWN", type = "minion", class = "druid", collectible = false, cost = 2, attack = 1, health = 2, tags = { "beast" }, keywords = { "poisonous" } },
    { id = "ICC_051b", rarity = "rare", name = "Scarab Form", text = "<b>Taunt</b>", set = "ICECROWN", type = "minion", class = "druid", collectible = false, cost = 2, attack = 1, health = 5, tags = { "beast" }, keywords = { "taunt" } },
    { id = "ICC_051t", rarity = "rare", name = "Druid of the Swarm", text = "<b>Poisonous</b>", set = "ICECROWN", type = "minion", class = "druid", collectible = false, cost = 2, attack = 1, health = 2, tags = { "beast" }, keywords = { "poisonous" } },
    { id = "ICC_051t2", rarity = "rare", name = "Druid of the Swarm", text = "<b>Taunt</b>", set = "ICECROWN", type = "minion", class = "druid", collectible = false, cost = 2, attack = 1, health = 5, tags = { "beast" }, keywords = { "taunt" } },
    { id = "ICC_051t3", rarity = "rare", name = "Druid of the Swarm", text = "<b>Taunt</b>\n<b>Poisonous</b>", set = "ICECROWN", type = "minion", class = "druid", collectible = false, cost = 2, attack = 1, health = 5, tags = { "beast" }, keywords = { "taunt", "poisonous" } },
}

return card
