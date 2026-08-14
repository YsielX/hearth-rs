local card = {
    api_version = 1, id = "ICC_065", name = "Bone Baron",
    text = "<b>Deathrattle:</b> Add two 1/1 Skeletons to your hand.",
    set = "ICECROWN", type = "minion", class = "rogue", rarity = "common",
    cost = 5, attack = 5, health = 5, tags = { "undead" }, keywords = { "deathrattle" },
}

function card.on_deathrattle(ctx, self)
    local player = ctx:controller(self)
    ctx:give_card(player, "ICC_026t")
    ctx:give_card(player, "ICC_026t")
end

return card
